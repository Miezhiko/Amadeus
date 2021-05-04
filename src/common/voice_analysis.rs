use crate::{
  common::voice_decoder::*,
  common::voice::*,
  common::voice_to_text::*
};

use serenity::prelude::Context;
use serenity::{async_trait, model::webhook::Webhook, prelude::RwLock};

use songbird::{
  driver::DecodeMode,
  model::{
    id::UserId,
    payload::{ClientConnect, ClientDisconnect, Speaking},
  },
  Event, EventContext, EventHandler as VoiceEventHandler,
};

use std::{collections::HashMap, process::Stdio, sync::Arc};
use tokio::{io::AsyncWriteExt, process::Command, task};
use uuid::Uuid;

#[derive(Clone)]
pub struct Receiver {
  ssrc_map: Arc<RwLock<HashMap<u32, UserId>>>,
  audio_buffer: Arc<RwLock<HashMap<u32, Vec<i16>>>>,
  encoded_audio_buffer: Arc<RwLock<HashMap<u32, Vec<i16>>>>,
  decoders: Arc<RwLock<HashMap<u32, Decoder>>>,
  webhook: Arc<Webhook>,
  context: Arc<Context>,
}

impl Receiver {
  pub fn new(webhook: Webhook, context: Arc<Context>) -> Self {
    // You can manage state here, such as a buffer of audio packet bytes so
    // you can later store them in intervals.
    let ssrc_map = Arc::new(RwLock::new(HashMap::new()));
    let audio_buffer = Arc::new(RwLock::new(HashMap::new()));
    let encoded_audio_buffer = Arc::new(RwLock::new(HashMap::new()));
    let decoders = Arc::new(RwLock::new(HashMap::new()));
    let webhook = Arc::new(webhook);
    Self {
      ssrc_map,
      audio_buffer,
      encoded_audio_buffer,
      decoders,
      webhook,
      context,
    }
  }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
  #[allow(unused_variables)]
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
    use songbird::EventContext as Ctx;

    match ctx {
      Ctx::SpeakingStateUpdate(Speaking {
        speaking,
        ssrc,
        user_id,
        ..
      }) => {
        // Discord voice calls use RTP, where every sender uses a randomly allocated
        // *Synchronisation Source* (SSRC) to allow receivers to tell which audio
        // stream a received packet belongs to. As this number is not derived from
        // the sender's user_id, only Discord Voice Gateway messages like this one
        // inform us about which random SSRC a user has been allocated. Future voice
        // packets will contain *only* the SSRC.
        //
        // You can implement logic here so that you can differentiate users'
        // SSRCs and map the SSRC to the User ID and maintain this state.
        // Using this map, you can map the `ssrc` in `voice_packet`
        // to the user ID and handle their audio packets separately.
        info!(
          "Speaking state update: user {:?} has SSRC {:?}, using {:?}",
          user_id, ssrc, speaking,
        );
        if let Some(user_id) = user_id {
          let mut map = self.ssrc_map.write().await;
          map.insert(*ssrc, *user_id);
          match DECODE_TYPE {
            DecodeMode::Decrypt => {
              {
                let mut audio_buf = self.encoded_audio_buffer.write().await;
                audio_buf.insert(*ssrc, Vec::new());
              }
              {
                let mut decoders = self.decoders.write().await;
                decoders.insert(*ssrc, Decoder::new());
              }
            }
            DecodeMode::Decode => {
              let mut audio_buf = self.audio_buffer.write().await;
              audio_buf.insert(*ssrc, Vec::new());
            }
            _ => {
              panic!("No supported decode mode found!")
            }
          }
        } // otherwise just ignore it since we can't do anything about that
      }
      Ctx::SpeakingUpdate { ssrc, speaking } => {
        // You can implement logic here which reacts to a user starting
        // or stopping speaking.
        let uid: u64 = {
          let map = self.ssrc_map.read().await;
          match map.get(ssrc) {
            Some(u) => u.0,
            None => 0,
          }
        };
        if !*speaking {
          let audio = match DECODE_TYPE {
            DecodeMode::Decrypt => {
              {
                let mut decoders = self.decoders.write().await;
                decoders.insert(*ssrc, Decoder::new());
              }
              {
                let mut buf = self.encoded_audio_buffer.write().await;
                match buf.insert(*ssrc, Vec::new()) {
                  Some(a) => a,
                  None => {
                    warn!(
                      "Didn't find a user with SSRC {} in the audio buffers.",
                      ssrc
                    );
                    return None;
                  }
                }
              }
            }
            DecodeMode::Decode => {
              let mut buf = self.audio_buffer.write().await;
              match buf.insert(*ssrc, Vec::new()) {
                Some(a) => a,
                None => {
                  warn!(
                    "Didn't find a user with SSRC {} in the audio buffers.",
                    ssrc
                  );
                  return None;
                }
              }
            }
            _ => {
              error!("Decode mode is invalid!");
              return None;
            }
          };
          // all of this code reeks of https://www.youtube.com/watch?v=lIFE7h3m40U
          let file_id = Uuid::new_v4();
          let file_path = format!("{}.wav", file_id.as_u128());

          let args = [
            "-f",
            "s16le",
            "-ar",
            "48000",
            "-ac",
            "2",
            "-i",
            "-",
            "-ac",
            "1",
            &file_path,
          ];

          let mut child = match Command::new("ffmpeg")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
          {
            Err(e) => {
              error!("Failed to spawn Sox!");
              return None;
            }
            Ok(c) => {
              info!("Spawned Sox!");
              c
            }
          };

          match child.stdin {
            Some(ref mut stdin) => {
              for i in audio {
                if let Err(e) = stdin.write_i16(i).await {
                  error!("Failed to write byte to ffmpeg stdin! {}", e);
                };
              }
            }
            None => {
              error!("Failed to open ffmpeg stdin!");
              return None;
            }
          };
          // we now have a file named "{}.wav" where {} is a random UUID as a 128-bit integer.
          // we should yield now to let other tasks proceed
          task::yield_now().await;
          let webhook = self.webhook.clone();
          let context = self.context.clone();

          task::spawn(async move {
            match child.wait().await {
              Ok(_) => {
                match run_stt(file_path.clone()).await {
                  Ok(r) => {
                    if r.len() != 0 {
                      match context.cache.user(uid).await {
                        Some(u) => {
                          let profile_picture = match u.avatar {
                            Some(a) => {
                              format!("https://cdn.discordapp.com/avatars/{}/{}.png", u.id, a)
                            }
                            None => u.default_avatar_url(),
                          };
                          let name = u.name;

                          let _ = webhook
                            .execute(&context, false, |m| {
                              m.avatar_url(profile_picture)
                                .content(r)
                                .username(name)
                            })
                            .await;
                          // see comments below
                        }
                        None => {}
                      }
                    }
                  }
                  Err(e) => {
                    error!("Failed to run speech-to-text! {}", e);
                  }
                };
              }
              Err(e) => {
                error!("ffmpeg failed! {}", e);
              }
            };
            if let Err(e) = tokio::fs::remove_file(&file_path).await {
              error!("Failed to delete {}! {}", &file_path, e);
            };
          });
        }
        info!(
          "Source {} (ID {}) has {} speaking.",
          ssrc,
          uid,
          if *speaking { "started" } else { "stopped" },
        );
      }
      Ctx::VoicePacket {
        audio,
        packet,
        payload_offset,
        payload_end_pad,
      } => {
        // An event which fires for every received audio packet,
        // containing the decoded data.

        let uid: u64 = {
          // block that will drop lock when exited
          let map = self.ssrc_map.read().await;
          match map.get(&packet.ssrc) {
            Some(u) => u.to_string().parse().unwrap(),
            None => 0,
          }
        };
        match audio {
          Some(audio) => {
            let mut buf = self.audio_buffer.write().await;
            let b = match buf.get_mut(&packet.ssrc) {
              Some(b) => b,
              None => {
                return None;
              }
            };
            b.extend(audio);
          }
          _ => {
            let mut audio = {
              let mut decoders = self.decoders.write().await;
              let decoder = match decoders
                .get_mut(&packet.ssrc) {
                Some(d) => d,
                None => {return None;}
              };
              let mut v = Vec::new();
              match decoder.opus_decoder.decode(&packet.payload, &mut v, false) {
                Ok(s) => {
                  info!("Decoded {} opus samples", s);
                }
                Err(e) => {
                  error!("Failed to decode opus: {}", e);
                  return None;
                }
              };
              v
            };
            let mut buf = self.encoded_audio_buffer.write().await;
            if let Some(b) = buf.get_mut(&packet.ssrc) {
              b.append(&mut audio);
            };
          }
        }
      }
      Ctx::RtcpPacket {
        packet,
        payload_offset,
        payload_end_pad,
      } => {
        // An event which fires for every received rtcp packet,
        // containing the call statistics and reporting information.
        // Probably ignorable for our purposes.
      }
      Ctx::ClientConnect(ClientConnect {
        audio_ssrc,
        video_ssrc,
        user_id,
        ..
      }) => {
        // You can implement your own logic here to handle a user who has joined the
        // voice channel e.g., allocate structures, map their SSRC to User ID.
        {
          // block that will drop the lock when exited
          let mut map = self.ssrc_map.write().await;
          map.insert(*audio_ssrc, *user_id);
        }
        info!(
          "Client connected: user {:?} has audio SSRC {:?}, video SSRC {:?}",
          user_id, audio_ssrc, video_ssrc,
        );
      }
      Ctx::ClientDisconnect(ClientDisconnect { user_id, .. }) => {
        // You can implement your own logic here to handle a user who has left the
        // voice channel e.g., finalise processing of statistics etc.
        // You will typically need to map the User ID to their SSRC; observed when
        // speaking or connecting.
        let key: Option<u32> = {
          let map = self.ssrc_map.read().await;
          let mut id: Option<u32> = None;
          for i in map.iter() {
            // walk the map to find the UserId
            if i.1 == user_id {
              id = Some(*i.0);
              break;
            }
          }
          id
        };
        if let Some(u) = key {
          match DECODE_TYPE {
            DecodeMode::Decrypt => {
              let mut audio_buf = self.encoded_audio_buffer.write().await;
              audio_buf.remove(&u);
            }
            DecodeMode::Decode => {
              let mut audio_buf = self.audio_buffer.write().await;
              audio_buf.remove(&u);
            }
            _ => {
              unimplemented!("No supported decode mode found!")
            }
          }
          let mut map = self.ssrc_map.write().await;
          map.remove(&u);
        };

        info!("Client disconnected: user {:?}", user_id);
      }
      _ => {
        // We won't be registering this struct for any more event classes.
        unimplemented!()
      }
    }

    None
  }
}

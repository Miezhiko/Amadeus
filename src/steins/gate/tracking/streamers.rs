use crate::{
  types::{ options::IOptions
         , tracking::{ TrackingGame, GameMode }
         , twitch::Twitch
         , goodgame::GoodGameData },
  collections::team::{ ALL, DISCORDS },
  common::{
    constants::{ LIVE_ROLE
               , STREAM_PICS }
  }
};

use serenity::{
  prelude::*,
  http::AttachmentType,
  model::id::{ ChannelId
             , GuildId }
};

use std::{
  borrow::Cow,
  collections::HashMap,
  time,
  sync::Arc
};

use chrono::DateTime;
use rand::Rng;

async fn clear_channel(channel: ChannelId, ctx: &Context) {
  if let Ok(vec_msg) = channel.messages(&ctx, |g| g.limit(25)).await {
    let mut vec_id = Vec::new();
    for message in vec_msg {
      for embed in message.embeds {
        if let Some(title) = embed.title {
          if title != "FINISHED" {
            vec_id.push(message.id);
            break;
          }
        }
      }
    }
    if !vec_id.is_empty() {
      match channel.delete_messages(&ctx, vec_id.as_slice()).await {
        Ok(nothing)  => nothing,
        Err(err) => warn!("Failed to clean live messages {}", err),
      };
    }
  }
}

#[allow(clippy::branches_sharing_code)]
pub async fn activate_streamers_tracking(
                     ctx:       &Arc<Context>
                   , options:   &IOptions
                   , token:     String ) {

  set!{ ctx_clone     = Arc::clone(ctx)
      , options_clone = options.clone() };

  for (d, df) in DISCORDS.iter() {
    if let Some(sc) = df.streams {
      clear_channel(ChannelId(sc), ctx).await;
    }
    let guild_id = GuildId(*d);
    if let Ok(g) = guild_id.to_partial_guild(&ctx).await {
      for p in ALL.iter() {
        if p.discords.contains(d) {
          if let Ok(mut m) = g.member(&ctx.http, p.player.discord).await {
            if let Some(r) = g.role_by_name(LIVE_ROLE) {
              if m.roles.contains(&r.id) {
                if let Err(why) = m.remove_role(&ctx, r).await {
                  error!("Failed to remove live streaming role {:?} on seerver {:?}", why, d);
                }
              }
            }
          }
        }
      }
    }
  }

  tokio::spawn(async move {
    let mut streams: HashMap<u64, TrackingGame> = HashMap::new();
    loop {
      let mut k_to_del: Vec<u64> = Vec::new();
      for (k, track) in streams.iter_mut() {
        if track.passed_time < (60 * 24) {
          // for real it's 60 + some time for processing
          track.passed_time += 1;
        } else {
          k_to_del.push(*k);
        }
      }
      for ktd in k_to_del {
        warn!("stream {} out with timeout", ktd);
        streams.remove(&ktd);
      }
      trace!("streams check");
      for p in ALL.iter() {
        if let Ok(user) = ctx_clone.http.get_user(p.player.discord).await {
          setm!{ twitch_live        = false
               , additional_fields  = Vec::new()
               , title              = String::new()
               , image              = None
               , em_url             = None };
          if p.player.streams.is_some() && !user.bot {
            let streams = p.player.streams.clone().unwrap();
            if streams.twitch.is_some() {
              let client = reqwest::Client::new();
              let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", &streams.twitch.unwrap());
              if let Ok(res) = client
                .get(&getq)
                .header("Authorization", token.clone())
                .header("Client-ID", options_clone.twitch_client_id.clone())
                .send().await {
                match res.json::<Twitch>().await {
                  Ok(t) => {
                    if !t.data.is_empty() {
                      let twd = &t.data[0];
                      let url = format!("https://www.twitch.tv/{}", twd.user_name);
                      let pic = twd.thumbnail_url.replace("{width}", "800")
                                                 .replace("{height}", "450");
                      if twd.type_string == "live" {
                        let start =
                          if let Ok(tws) = DateTime::parse_from_rfc3339(&twd.started_at) {
                            // maybe MSK time?
                            let cet_time = tws.with_timezone(&chrono_tz::CET).time();
                            let time_format = "%k:%M";
                            cet_time.format(time_format).to_string()
                          } else {
                            twd.started_at.clone()
                          };
                        let t_d = format!("{}\n{}\nviewers: {}\nstarted: {} CET",
                                    twd.title, url, twd.viewer_count, start);
                        additional_fields.push(("Live on twitch", t_d, true));
                        title       = twd.title.clone();
                        image       = Some(pic);
                        em_url      = Some(url);
                        twitch_live = true;
                      }
                    }
                  }, Err(why) => {
                    // try again to get debug information
                    if let Ok(res) = client
                      .get(&getq)
                      .header("Authorization", token.clone())
                      .header("Client-ID", options_clone.twitch_client_id.clone())
                      .send().await {
                      if let Ok(some_text) = res.text().await {
                        error!("Failed to parse twitch structs\nrequest: {}\nerror: {:?}\ntext: {}", &getq, why, some_text);
                      }
                    } else {
                      error!("Failed to parse twitch structs\nrequest: {}\nerror: {:?}", &getq, why);
                    }
                  }
                }
              }
            }
            if streams.ggru.is_some() {
              let ggru = streams.ggru.clone().unwrap();
              let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{}", &ggru);
              if let Ok(gg) = reqwest::get(&ggru_link).await {
                match gg.json::<GoodGameData>().await {
                  Ok(ggdata) => {
                    if ggdata.status == "Live" {
                      let url = format!("https://goodgame.ru/channel/{}", &ggru);
                      if twitch_live {
                        let viewers = format!( "viewers: {}\nin chat: {}"
                                             , ggdata.viewers, ggdata.users_in_chat );
                        let titurl =
                          format!("{}\n{}\n{}", &ggdata.channel.title, url, viewers);
                        additional_fields.push(("Live on ggru", titurl, true));
                      } else {
                        let viewers = format!( "viewers: {}\nin chat: {}"
                                             , ggdata.viewers, ggdata.users_in_chat );
                        additional_fields.push(("Live on ggru", viewers, true));
                        title  = ggdata.channel.title.clone();
                        let img_gg =
                          if ggdata.channel.thumb.starts_with("//") {
                            String::from("https:") + &ggdata.channel.thumb
                          } else {
                            ggdata.channel.thumb.clone()
                          };
                        image  = Some(img_gg);
                        em_url = Some(url);
                      }
                    }
                  }, Err(why) => {
                    error!("Failed to parse good game structs {:?} on request {}", why, &ggru_link);
                  }
                };
              }
            }
          } else { continue; }

          if !additional_fields.is_empty() {
            if title.is_empty() {
              title = String::from("LIVE");
            }
            if let Some(track) = streams.get(&p.player.discord) {

              for t_msg in &track.tracking_msg_id {
              if let Some(discord) = DISCORDS.get(&t_msg.0) {
              if let Some(streams_channel) = discord.streams {

              if let Ok(mut msg) = ctx_clone.http.get_message( streams_channel
                                                             , t_msg.1 ).await {
                let footer = if track.passed_time > 60 {
                    let hours: u32 = track.passed_time / 60;
                    let minutes = track.passed_time % 60;
                    format!("Passed: {} hours {} min", hours, minutes)
                  } else {
                    format!("Passed: {} min", track.passed_time)
                  };
                let mut fields = Vec::new();
                let mut img = None;
                let mut url = None;
                let mut color = (32, 32, 32);
                if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                  for f in msg.embeds[0].fields.clone() {
                    fields.push((f.name, f.value, f.inline));
                  }
                  img   = msg.embeds[0].image.clone();
                  url   = msg.embeds[0].url.clone();
                  color = msg.embeds[0].colour.tuple();
                };
                let is_now_live = format!("{} is now live!", &user.name);
                if let Err(why) = msg.edit(&ctx_clone.http, |m| m
                  .embed(|e|  {
                    let mut e = e
                      .title(&title)
                      .colour(color)
                      .author(|a| a.icon_url(&user.face()).name(&is_now_live))
                      .footer(|f| f.text(&footer));
                    if !fields.is_empty() {
                      e = e.fields(fields.clone());
                    }
                    if let Some(some_img) = &img {
                      e = e.image(some_img.url.clone());
                    }
                    if let Some(some_url) = &url {
                      e = e.url(some_url);
                    }
                    e
                  }
                )).await {
                  error!("Failed to edit stream msg {:?}", why);
                }
              }

              } // if with a stream channel
              } // if let some discord
              } // for discord servers

            } else {
              let is_now_live = format!("{} started stream!", &user.name);
              // stream thumbnail image caching only work with twitch
              // (most likely because of image format difference jpg/png)
              if twitch_live {
                if let Some(some_image) = &image {
                  if let Ok(response) = reqwest::get(some_image.as_str()).await {
                    if let Ok(bytes) = response.bytes().await {
                      let cow = AttachmentType::Bytes {
                        data: Cow::from(bytes.as_ref()),
                        filename: "stream_img.jpg".to_string()
                      };
                      match STREAM_PICS.send_message(&ctx_clone, |m| m.add_file(cow)).await {
                        Ok(msg) => {
                          if !msg.attachments.is_empty() {
                            let img_attachment = &msg.attachments[0];
                            image = Some(img_attachment.url.clone());
                          }
                        },
                        Err(why) => {
                          error!("Failed to download and post stream img {:?}", why);
                        }
                      };
                    }
                  }
                }
              }
              set! { red   = rand::thread_rng().gen_range(0..255)
                   , green = rand::thread_rng().gen_range(0..255)
                   , blue  = rand::thread_rng().gen_range(0..255) };

              for d in &p.discords {
              if let Some(ds) = DISCORDS.get(d) {

              let discord_guild = GuildId(*d);
              if let Ok(guild) = discord_guild.to_partial_guild(&ctx_clone).await {
                if let Ok(mut member) = guild.member(&ctx_clone.http, user.id).await {
                  if let Some(role) = guild.role_by_name(LIVE_ROLE) {
                    if !member.roles.contains(&role.id) {
                      if let Err(why) = member.add_role(&ctx_clone, role).await {
                        error!("Failed to assign live streaming role {:?}", why);
                      }
                    }
                  }
                }
              }

              if let Some(sc) = ds.streams {

              match ChannelId(sc).send_message(&ctx_clone, |m| m
                .embed(|e| {
                  let mut e = e
                    .title(&title)
                    .colour((red, green, blue))
                    .author(|a| a.icon_url(&user.face()).name(&is_now_live));
                  if !additional_fields.is_empty() {
                    e = e.fields(additional_fields.clone());
                  }
                  if let Some(some_image) = &image {
                    e = e.image(some_image);
                  }
                  if let Some(some_url) = &em_url {
                    e = e.url(some_url);
                  }
                  e
                }
              )).await {
                Ok(msg_id) => {
                  let playa_for_stream = p.clone();
                  if let Some(inserted) = streams.get_mut(&playa_for_stream.player.discord) {
                    if !inserted.tracking_msg_id.contains(&(*d, msg_id.id.0)) {
                      inserted.tracking_msg_id.push((*d, msg_id.id.0));
                    }
                  } else {
                    streams.insert(playa_for_stream.player.discord, TrackingGame {
                      // TODO: get discord from player
                      tracking_msg_id: vec![(*d, msg_id.id.0)],
                      passed_time: 0,
                      still_live: true,
                      players: vec![playa_for_stream], bets: vec![], fails: 0,
                      mode: GameMode::Solo }
                    );
                  }
                },
                Err(why) => {
                  error!("Failed to post stream {:?}", why);
                }
              }

              } else { // if no stream channel
                let playa_for_stream = p.clone();
                if streams.get(&playa_for_stream.player.discord).is_none() {
                  streams.insert(playa_for_stream.player.discord, TrackingGame {
                    tracking_msg_id: vec![],
                    passed_time: 0,
                    still_live: true,
                    players: vec![playa_for_stream], bets: vec![], fails: 0,
                    mode: GameMode::Solo }
                  );
                }
              }

              } // if there are fields
              } // discords for

            }
          } else if let Some(track) = streams.get(&p.player.discord) {
            // stream finished

            for t_msg in &track.tracking_msg_id {
            if let Some(discord) = DISCORDS.get(&t_msg.0) {

            let discord_guild = GuildId(t_msg.0);
            if let Ok(guild) = discord_guild.to_partial_guild(&ctx_clone).await {
              if let Ok(mut member) = guild.member(&ctx_clone.http, user.id).await {
                if let Some(role) = guild.role_by_name(LIVE_ROLE) {
                  if member.roles.contains(&role.id) {
                    if let Err(why) = member.remove_role(&ctx_clone, role).await {
                      error!("Failed to remove live streaming role {:?}", why);
                    }
                  }
                }
              }
            }

            if let Some(streas_channel) = discord.streams {

            if let Ok(mut msg) = ctx_clone.http.get_message( streas_channel
                                                           , t_msg.1 ).await {
              let footer = if track.passed_time > 60 {
                  let hours: u32 = track.passed_time / 60;
                  let minutes = track.passed_time % 60;
                  format!("Passed: {} hours {} min", hours, minutes)
                } else {
                  format!("Passed: {} min", track.passed_time)
                };
              let mut fields = Vec::new();
              let mut img = None;
              let mut url = None;
              let mut color = (32, 32, 32);
              if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                for f in msg.embeds[0].fields.clone() {
                  fields.push((f.name, f.value, f.inline));
                }
                img   = msg.embeds[0].image.clone();
                url   = msg.embeds[0].url.clone();
                color = msg.embeds[0].colour.tuple();
              };
              if let Err(why) = msg.edit(&ctx_clone.http, |m| m
                .embed(|e| {
                  let mut e = e
                    .title("FINISHED")
                    .colour(color)
                    .author(|a| a.icon_url(&user.face()).name(&user.name))
                    .footer(|f| f.text(&footer));
                  if !fields.is_empty() {
                    e = e.fields(fields.clone());
                  }
                  if let Some(some_img) = &img {
                    e = e.image(some_img.url.clone());
                  }
                  if let Some(some_url) = &url {
                    e = e.url(some_url);
                  }
                  e
                }
              )).await {
                error!("Failed to edit stream msg {:?}", why);
              }
            }

            } // if with a stream channel
            } // if let some discord
            } // for discord servers

            streams.remove(&p.player.discord);
          }
        }
        tokio::time::sleep(time::Duration::from_millis(200)).await;
      }
      /* every minute */
      tokio::time::sleep(time::Duration::from_secs(60)).await;
    }
  });
}

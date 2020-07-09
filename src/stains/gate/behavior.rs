use crate::{
  common::types::AOptions,
  stains::{
    ai::chain,
    cyber, cyber::types::TrackingGame,
  }
};

use serenity::{
  prelude::*,
  model::{
    id::GuildId, id::ChannelId,
    gateway::Activity
  }
};

use std::{
  sync::atomic::Ordering,
  time
};

use rand::Rng;

use futures_util::stream::{self, StreamExt};

pub async fn activate(ctx: &Context, options: &AOptions) {
  let last_guild_u64 = options.last_guild.parse::<u64>().unwrap_or(0);
  if last_guild_u64 != 0 {
    let guild_id = GuildId( last_guild_u64 );
    if let Ok(channels) = guild_id.channels(ctx).await {

      let mut background_threads_successfully_started = false;
      let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
      ctx.set_activity(Activity::listening(version.as_str())).await;
      ctx.idle().await;

      let main_channels = stream::iter(channels.iter())
      .filter_map(|(c, _)| async move {
        if let Some(name) = c.name(&ctx).await {
          if name == "main" { Some(c) } else { None }
        } else { None }
      }).collect::<Vec<&ChannelId>>().await;
      if main_channels.len() > 0 {
        let channel = main_channels[0];
        set!{ ch_clone = channel.clone()
            , ctx_clone = ctx.clone() };
        tokio::spawn(async move {
          loop {
            let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
            let rndx = rand::thread_rng().gen_range(0, activity_level);
            if rndx == 1 {
              let ai_text = chain::generate_english_or_russian(&ctx_clone, &guild_id, 9000).await;
              if let Err(why) = ch_clone.send_message(&ctx_clone, |m| {
                m.content(ai_text)
              }).await {
                error!("Failed to post periodic message {:?}", why);
              }
            }
            std::thread::sleep(time::Duration::from_secs(30*60));
          }
        });
      }

      let log_channels = stream::iter(channels.iter())
      .filter_map(|(c, _)| async move {
        if let Some(name) = c.name(&ctx).await {
          if name == "log" { Some(c) } else { None }
        } else { None }
      }).collect::<Vec<&ChannelId>>().await;
      if log_channels.len() > 0 {
        let channel = log_channels[0];

        // Delete live games from log channel (if some)
        for vec_msg in channel.messages(&ctx, |g| g.limit(50)).await {
          let mut vec_id = Vec::new();
          for message in vec_msg {
            for embed in message.embeds {
              if let Some(title) = embed.title {
                if title == "LIVE" || title == "JUST STARTED" {
                  vec_id.push(message.id);
                  break;
                }
              }
            }
          }
          match channel.delete_messages(&ctx, vec_id.as_slice()).await {
            Ok(nothing)  => nothing,
            Err(err) => warn!("Failed to clean live messages {}", err),
          };
        }

        set!{ ch_clone = channel.clone(),
              ctx_clone = ctx.clone(),
              ch_ud = ch_clone.as_u64().clone(),
              options_clone = options.clone() };
        tokio::spawn(async move {
          loop {
            let mut games_lock = cyber::team_checker::GAMES.lock().await; {
              let mut k_to_del : Vec<String> = Vec::new();
              for (k, track) in games_lock.iter_mut() {
                if track.passed_time < 666 {
                  track.passed_time += 1;
                  track.still_live = false;
                } else {
                  k_to_del.push(k.clone());
                }
              }
              for ktd in k_to_del {
                warn!("match {} out with timeout", ktd);
                games_lock.remove(ktd.as_str());
              }
            }
            info!("check");
            if !background_threads_successfully_started {
              ctx_clone.set_activity(Activity::playing(version.as_str())).await;
              ctx_clone.online().await;
            }
            background_threads_successfully_started = true;
            let our_gsx = cyber::team_checker::check(&ctx_clone, ch_ud).await;
            for game in our_gsx {
              set!{ game_key = game.key.clone()
                  , discord_user = game.user };
              if let Ok(user) = ctx_clone.http.get_user(discord_user).await {

                let mut twitch_live = false;
                let mut additional_fields = Vec::new();
                let mut image : Option<String> = None;
                let mut em_url : Option<String> = None;

                if game.stream.is_some() {
                  set! { streams = game.stream.clone().unwrap()
                       , twitch = &streams.twitch
                       , ggru = &streams.ggru };

                  if twitch.is_some() {
                    let client = reqwest::Client::new();
                    let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", twitch.unwrap());
                    if let Ok(res) = client
                      .get(getq.as_str())
                      .header("Authorization", options_clone.twitch_oauth.clone())
                      .header("Client-ID", options_clone.twitch_client_id.clone())
                      .send().await {
                      match res.json::<cyber::twitch::Twitch>().await {
                        Ok(t) => {
                          if t.data.len() > 0 {
                            let d = &t.data[0];
                            let url = format!("https://www.twitch.tv/{}", d.user_name);
                            let pic = d.thumbnail_url.replace("{width}", "800")
                                                     .replace("{height}", "450");
                            if d.type_string == "live" {
                              additional_fields.push(("Live on twitch", d.title.clone(), false));
                              image = Some(pic);
                              em_url = Some(url);
                              twitch_live = true;
                            }
                          }
                        }, Err(why) => {
                          error!("Failed to parse twitch structs {:?}", why);
                        }
                      }
                    }
                  }

                  if ggru.is_some() {
                    let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{}", ggru.unwrap());
                    if let Ok(gg) = reqwest::get(ggru_link.as_str()).await {
                      match gg.json::<cyber::goodgame::GoodGameData>().await {
                        Ok(ggdata) => {
                          if ggdata.status == "Live" {
                            let url = format!("https://goodgame.ru/channel/{}", ggru.unwrap());
                            if twitch_live {
                              let titurl =
                                format!("{}\n{}", ggdata.channel.title.as_str(), url);
                              additional_fields.push(("Live on ggru", titurl, false));
                            } else {
                              additional_fields.push(("Live on ggru", ggdata.channel.title.clone(), false));
                              image = Some(ggdata.channel.thumb.clone());
                              em_url = Some(url);
                            }
                          }
                        }, Err(why) => {
                          error!("Failed to parse good game structs {:?}", why);
                        }
                      };
                    }
                  }

                }

                match ch_clone.send_message(&ctx_clone, |m| m
                  .embed(|e| {
                    let mut e = e
                      .title("JUST STARTED")
                      .author(|a| a.icon_url(&user.face()).name(&user.name))
                      .description(game.description.as_str());
                    if additional_fields.len() > 0 {
                      e = e.fields(additional_fields);
                    }
                    if image.is_some() {
                      e = e.image(image.unwrap());
                    }
                    if em_url.is_some() {
                      e = e.url(em_url.unwrap());
                    }
                    e
                  }
                )).await {
                  Ok(msg_id) => {
                    let mut games_lock = cyber::team_checker::GAMES.lock().await; {
                      games_lock.insert(game_key, TrackingGame {
                        tracking_msg_id: msg_id.id.as_u64().clone(),
                        passed_time: 0,
                        still_live: false,
                        tracking_usr_id: discord_user }
                      );
                    }
                  },
                  Err(why) => {
                    error!("Failed to post live match {:?}", why);
                  }
                }
              }
            }
            std::thread::sleep(time::Duration::from_secs(30));
          }
        });
      }
    }
  }
}

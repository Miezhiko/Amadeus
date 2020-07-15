use crate::{
  common::types::IOptions,
  common::help::channel::channel_by_name,
  stains::{
    ai::chain,
    cyber, cyber::types::TrackingGame,
  },
  commands::pad::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    id::GuildId,
    gateway::Activity
  }
};

use std::{
  sync::atomic::Ordering,
  time
};

use rand::Rng;

pub async fn activate(ctx: &Context, options: &IOptions) {
  info!("activation has started");
  // set actual season for pad statistics
  update_current_season().await;

  if options.guild != 0 {
    let guild_id = GuildId( options.guild );

    // updating ai:chain cache
    chain::update_cache(&ctx, &guild_id).await;

    if let Ok(channels) = guild_id.channels(ctx).await {

      let mut background_threads_successfully_started = false;
      let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
      ctx.set_activity(Activity::listening(version.as_str())).await;
      ctx.idle().await;

      if let Some((channel, _)) = channel_by_name(&ctx, &channels, "main").await {
        set!{ ch_clone = channel.clone()
            , ctx_clone = ctx.clone() };
        tokio::spawn(async move {
          loop {
            let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
            let rndx = rand::thread_rng().gen_range(0, activity_level);
            if rndx == 1 {
              let ai_text = chain::generate_english_or_russian(&ctx_clone, &guild_id).await;
              if let Err(why) = ch_clone.send_message(&ctx_clone, |m| {
                m.content(ai_text)
              }).await {
                error!("Failed to post periodic message {:?}", why);
              }
            }
            update_current_season().await;
            /* every 30 minutes */
            tokio::time::delay_for(time::Duration::from_secs(30*60)).await;
          }
        });
      }

      if let Some((channel, _)) = channel_by_name(&ctx, &channels, "log").await {

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
          if vec_id.len() > 0 {
            match channel.delete_messages(&ctx, vec_id.as_slice()).await {
              Ok(nothing)  => nothing,
              Err(err) => warn!("Failed to clean live messages {}", err),
            };
          }
        }

        set!{ ch_clone = channel.clone(),
              ctx_clone = ctx.clone(),
              ch_ud = ch_clone.as_u64().clone(),
              options_clone = options.clone() };

        tokio::spawn(async move {
          let mut games_lock = cyber::team_checker::GAMES.lock().await;
          loop {
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
            info!("check");
            if !background_threads_successfully_started {
              ctx_clone.set_activity(Activity::playing(version.as_str())).await;
              ctx_clone.online().await;
            }
            background_threads_successfully_started = true;
            let our_gsx = cyber::team_checker::check(&ctx_clone, ch_ud, &mut games_lock).await;
            for game in our_gsx {
              let game_key = game.key.clone();
              if let Ok(user) = ctx_clone.http.get_user(game.player.discord).await {

                setm!{ twitch_live        = false
                     , additional_fields  = Vec::new()
                     , image              = None
                     , em_url             = None };

                if game.player.streams.is_some() {
                  let streams = game.player.streams.clone().unwrap();
                  if streams.twitch.is_some() {
                    let client = reqwest::Client::new();
                    let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", streams.twitch.unwrap().as_str());
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
                  if streams.ggru.is_some() {
                    let ggru = streams.ggru.clone().unwrap();
                    let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{}", ggru.as_str());
                    if let Ok(gg) = reqwest::get(ggru_link.as_str()).await {
                      match gg.json::<cyber::goodgame::GoodGameData>().await {
                        Ok(ggdata) => {
                          if ggdata.status == "Live" {
                            let url = format!("https://goodgame.ru/channel/{}", ggru.as_str());
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
                    games_lock.insert(game_key, TrackingGame {
                      tracking_msg_id: msg_id.id.as_u64().clone(),
                      passed_time: 0,
                      still_live: false,
                      player: game.player }
                    );
                  },
                  Err(why) => {
                    error!("Failed to post live match {:?}", why);
                  }
                }
              }
            }
            tokio::time::delay_for(time::Duration::from_secs(30)).await;
          }
        });
      }
    }
  }
}

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
    id::GuildId,
    gateway::Activity
  }
};

use std::{
  sync::atomic::Ordering,
  time
};

use rand::Rng;

pub fn activate(ctx: &Context, options: &AOptions) {
  let last_guild_u64 = options.last_guild.parse::<u64>().unwrap_or(0);
  if last_guild_u64 != 0 {
    let guild_id = GuildId( last_guild_u64 );
    if let Ok(channels) = guild_id.channels(&ctx) {

      let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
      &ctx.set_activity(Activity::listening(version.as_str()));
      &ctx.idle();

      let main_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx)
          { name == "main" } else { false });
      if let Some((_, channel)) = main_channel {
        set!{ ch_clone = channel.clone()
            , ctx_clone = ctx.clone() };
        std::thread::spawn(move || {
          loop {
            let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
            let rndx = rand::thread_rng().gen_range(0, activity_level);
            if rndx == 1 {
              if let Err(why) = ch_clone.send_message(&ctx_clone, |m| {
                let ai_text = chain::generate_english_or_russian(&ctx_clone, &guild_id, 9000);
                m.content(ai_text)
              }) {
                error!("Failed to post periodic message {:?}", why);
              }
            }
            std::thread::sleep(time::Duration::from_secs(30*60));
          }
        });
      }

      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx)
          { name == "log" } else { false });
      if let Some((_, channel)) = log_channel {
        set!{ ch_clone = channel.clone(),
              ctx_clone = ctx.clone(),
              ch_ud = ch_clone.id.as_u64().clone(),
              options_clone = options.clone() };
        std::thread::spawn(move || {
          loop {
            if let Ok(mut games_lock) = cyber::team_checker::GAMES.lock() {
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
            { info!("check");
              let our_gsx = cyber::team_checker::check(&ctx_clone, ch_ud);
              for game in our_gsx {
                if let Ok(user) = ctx_clone.http.get_user(game.user) {
                  match ch_clone.send_message(&ctx_clone, |m| m
                    .embed(|e| {
                      let mut e = e
                        .title("Just started")
                        .author(|a| a.icon_url(&user.face()).name(&user.name))
                        .description(game.description.as_str());
                      let (ggru, twitch) = &game.stream;
                      let mut twitch_live = false;
                      if twitch.is_some() {
                        let client = reqwest::blocking::Client::new();
                        let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", twitch.unwrap());
                        if let Ok(res) = client
                          .get(getq.as_str())
                          .header("Authorization", options_clone.twitch_oauth.clone())
                          .header("Client-ID", options_clone.twitch_client_id.clone())
                          .send() {
                          match res.json::<cyber::twitch::Twitch>() {
                            Ok(t) => {
                              if t.data.len() > 0 {
                                let d = &t.data[0];
                                let url = format!("https://www.twitch.tv/{}", d.user_name);
                                // still 800x400 feels very wide
                                let pic = d.thumbnail_url.replace("{width}", "800")
                                                        .replace("{height}", "500");
                                if d.type_string == "live" {
                                  e = e.fields(vec![("Live on twitch", d.title.clone(), false)])
                                      .image(pic)
                                      .url(url);
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
                        if let Ok(gg) = reqwest::blocking::get(ggru_link.as_str()) {
                          match gg.json::<cyber::goodgame::GoodGameData>() {
                            Ok(ggdata) => {
                              if ggdata.status == "Live" {
                                let url = format!("https://goodgame.ru/channel/{}", ggru.unwrap());
                                if twitch_live {
                                  let titurl =
                                    format!("{}\n{}", ggdata.channel.title.as_str(), url);
                                  e = e.fields(vec![("Live on ggru", titurl, false)]);
                                } else {
                                  e = e.fields(vec![("Live on ggru", ggdata.channel.title.clone(), false)])
                                      .image(ggdata.channel.thumb.clone())
                                      .url(url);
                                }
                              }
                            }, Err(why) => {
                              error!("Failed to parse good game structs {:?}", why);
                            }
                          };
                        }
                      }
                      e
                    }
                  )) {
                    Ok(msg_id) => {
                      if let Ok(mut games_lock) = cyber::team_checker::GAMES.lock() {
                        games_lock.insert(game.key, TrackingGame {
                          tracking_msg_id: msg_id.id.as_u64().clone(),
                          passed_time: 0,
                          still_live: false,
                          tracking_usr_id: game.user }
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
          }
        });
      }
    }
  }
}
use crate::{
  types::{ serenity::ReqwestClient
         , options::IOptions
         , tracking::{ TrackingGame
                     , Bet, GameMode }
         , twitch::{ Twitch, TWITCH_WC3 }
         , goodgame::GoodGameData
         },
  common::{ db::trees::points
          , aka },
  collections::team::DISCORDS,
  steins::warcraft::{ aka_checker::AKA
                    , poller::{ self, checker } }
};

use chrono::{ Timelike, Datelike, Weekday };
use serenity::{ prelude::*
              , builder::*
              , model::id::{ ChannelId, UserId }
              , model::channel::ReactionType };

use std::{ time
         , sync::{ Arc
                 , atomic::Ordering }
};

use rand::Rng;

const DAY_TIMEOUT_SECS: u32   = 60;
const NIGHT_TIMEOUT_SECS: u32 = 30;

const DAY_TIMEOUT: time::Duration   = time::Duration::from_secs(DAY_TIMEOUT_SECS as u64);
const NIGHT_TIMEOUT: time::Duration = time::Duration::from_secs(NIGHT_TIMEOUT_SECS as u64);

async fn clean_games_channel(channel: &ChannelId, ctx: &Context) {
  if let Ok(vec_msg) = channel.messages(&ctx, GetMessages::default().limit(50)).await {
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
    if !vec_id.is_empty() {
      match channel.delete_messages(&ctx, vec_id.as_slice()).await {
        Ok(nothing)  => nothing,
        Err(err) => warn!("Failed to clean live messages {err}"),
      };
    }
  }
}

pub async fn activate_games_tracking(
                     ctx:       &Arc<Context>
                   , options:   &Arc<IOptions>
                   , token:     String
                   , amadeus:   u64 ) {

  set!{ ctx_clone     = Arc::clone(ctx)
      , options_clone = Arc::clone(options) };

  { // AKA lock scope
    let mut aka_lock = AKA.lock().await;
    match aka::get_aka().await {
      Ok(aka) => { *aka_lock = aka; },
      Err(wa) => { warn!("something with aka.rs, {wa}"); }
    }
  }

  // Delete live games from log channel (if some)
  for (_, df) in DISCORDS.iter() {
    if let Some(sc) = df.games {
      let channel = ChannelId(to_nzu!(sc));
      clean_games_channel(&channel, ctx).await;
    }
    if let Some(sc) = df.games2 {
      let channel = ChannelId(to_nzu!(sc));
      clean_games_channel(&channel, ctx).await;
    }
    if let Some(sc) = df.games4 {
      let channel = ChannelId(to_nzu!(sc));
      clean_games_channel(&channel, ctx).await;
    }
  }

  tokio::spawn(async move {
    let rqcl = {
      set!{ data = ctx_clone.data.read().await
          , rqcl = data.get::<ReqwestClient>().unwrap() };
      rqcl.clone()
    };
    loop {

      { // scope for GAMES lock
        trace!("team games: clearing");
        let mut games_lock = poller::GAMES.lock().await;
        let mut k_to_del: Vec<String> = Vec::new();
        for (k, track) in games_lock.iter_mut() {
          if track.passed_time < 666 {
            track.passed_time += 1;
            track.still_live = false;
          } else {
            k_to_del.push(k.clone());
          }
        }
        for ktd in k_to_del {
          warn!("match {ktd} out with timeout");
          games_lock.remove(&ktd);
        }
      }

      let our_gsx = checker::check( &ctx_clone
                                  , options_clone.guild
                                  , &rqcl
                                  ).await;

      for game in our_gsx {
        let game_key = game.key.clone();
        let playa = &game.players[0];
        if let Ok(user) = ctx_clone.http
          .get_user( UserId(to_nzu!(playa.player.discord)) ).await {

          setm!{ twitch_live        = false
               , additional_fields  = vec![]
               , image              = None
               , em_url             = None };

          if let Some(streams) = &playa.player.streams {
            if let Some(twitch_stream) = &streams.twitch {
              let getq = format!("https://api.twitch.tv/helix/streams?user_login={twitch_stream}");
              if let Ok(res) = rqcl
                .get(&getq)
                .header("Authorization", token.clone())
                .header("Client-ID", options_clone.twitch_client_id.clone())
                .send().await {
                match res.json::<Twitch>().await {
                  Ok(t) => {
                    if !t.data.is_empty() {
                      let twd = &t.data[0];
                      if twd.game_id == TWITCH_WC3 {
                        let url = format!("https://www.twitch.tv/{}", twd.user_name);
                        let pic = twd.thumbnail_url.replace("{width}", "800")
                                                   .replace("{height}", "450");
                        if twd.type_string == "live" {
                          let titurl = format!("{}\n{url}", &twd.title);
                          additional_fields.push(("Live on twitch", titurl, false));
                          image       = Some(pic);
                          em_url      = Some(url);
                          twitch_live = true;
                        }
                      }
                    }
                  }, Err(why) => {
                    error!("Failed to parse twitch structs {why}");
                  }
                }
              }
            }
            if let Some(ggru) = &streams.ggru {
              let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{ggru}");
              if let Ok(gg) = rqcl.get(&ggru_link).send().await {
                match gg.json::<GoodGameData>().await {
                  Ok(ggdata) => {
                    if ggdata.status == "Live" {
                      let url = format!("https://goodgame.ru/channel/{ggru}");
                      if twitch_live {
                        let titurl =
                          format!("{}\n{url}", &ggdata.channel.title);
                        additional_fields.push(("Live on ggru", titurl, true));
                      } else {
                        let title = if ggdata.channel.title.is_empty() {
                            String::from("LIVE")
                          } else {
                            ggdata.channel.title.clone()
                          };
                        additional_fields.push(("Live on ggru", title, false));
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
                    error!("Failed to parse good game structs {why} on request {}", &ggru_link);
                  }
                };
              }
            }
          }

          set!{ red   = rand::thread_rng().gen_range(0..255)
              , green = rand::thread_rng().gen_range(0..255)
              , blue  = rand::thread_rng().gen_range(0..255) };

          let nickname_maybe = user.nick_in(&ctx_clone.http, options_clone.guild).await;
          let nick = nickname_maybe.unwrap_or_else(|| user.name.clone());

          for d in playa.discords.iter() {
            if let Some(ds) = DISCORDS.get(d) {

            let game_channel_maybe = match game.mode {
              GameMode::Solo  => ds.games,
              GameMode::Team2 => ds.games2,
              GameMode::Team4 => ds.games4
            };

            if let Some(gc) = game_channel_maybe {
            let game_channel = ChannelId(to_nzu!(gc));

            let mut e = CreateEmbed::new()
              .title("JUST STARTED")
              .author(CreateEmbedAuthor::new(&nick).icon_url(user.face()))
              .colour((red, green, blue));
            if !game.description.is_empty() {
              e = e.description(&game.description[0]);
              if game.description.len() > 2 {
                let d_fields = vec![
                  ("Team 1", game.description[1].as_str(), true)
                , ("Team 2", game.description[2].as_str(), true)
                , (&game.host, "\u{200b}", false)
                ];
                e = e.fields(d_fields);
              }
            }
            if !additional_fields.is_empty() {
              e = e.fields(additional_fields.clone());
            }
            if let Some(some_image) = &image {
              e = e.image(some_image);
            }
            if let Some(some_url) = &em_url {
              e = e.url(some_url);
            }
            match game_channel.send_message(&ctx_clone, CreateMessage::new()
              .embed(e)
            ).await {
              Ok(msg_id) => {
                { // scope for games_lock
                  trace!("team games: starting");
                  let mut games_lock = poller::GAMES.lock().await;
                  if let Some(inserted) = games_lock.get_mut(&game_key) {
                    if !inserted.tracking_msg_id.contains(&(*d, msg_id.id.0.get())) {
                      inserted.tracking_msg_id.push((*d, msg_id.id.0.get()));
                    }
                  } else {
                    games_lock.insert( game_key.clone()
                      , TrackingGame { tracking_msg_id: vec![(*d, msg_id.id.0.get())]
                                     , passed_time: 0
                                     , still_live: false
                                     , players: game.players.clone().into_iter()
                                                            .cloned().collect()
                                     , bets: vec![]
                                     , fails: 0
                                     , mode: game.mode, flo_tv: None } );
                  }
                }
                let up = ReactionType::Unicode(String::from("👍🏻"));
                let dw = ReactionType::Unicode(String::from("👎🏻"));
                let _ = msg_id.react(&ctx_clone.http, up).await;
                let _ = msg_id.react(&ctx_clone.http, dw).await;
                // run thread inside thread for reactions
                // we're cloning ctx yet another time here!
                let xtx_clone = Arc::clone(&ctx_clone);
                let game_key_clone = game_key.clone();
                tokio::spawn(async move {
                  loop {
                    // 10 minutes for each game
                    let collector = msg_id.await_reaction(&xtx_clone.shard)
                                          .timeout(time::Duration::from_secs(600));
                    if let Some(reaction) = collector.await {
                      let emoji = &reaction.emoji;
                      if let Some(u) = reaction.user_id {
                        if let Some(g) = reaction.guild_id {
                          if let Ok(p) = points::get_points( g.0.get(), u.0.get() ).await {
                            if p > 100 {
                              if let ReactionType::Unicode(emoji_data) = emoji {
                                if emoji_data == "👍🏻" || emoji_data == "👎🏻" {
                                  let is_positive = emoji_data == "👍🏻";
                                  { // games lock scope
                                    trace!("team games: thumb was clicked");
                                    let mut gl = poller::GAMES.lock().await;
                                    if let Some(track) = gl.get_mut(&game_key_clone) {
                                      if track.still_live {
                                        // you bet only once
                                        if !track.bets.iter().any(|b| b.member == u.0.get()) {
                                          let bet = Bet { guild: g.0.get()
                                                        , member: u.0.get()
                                                        , points: 100
                                                        , positive: is_positive
                                                        , registered: false };
                                          let (succ, rst) = points::give_points( g.0.get(), u.0.get()
                                                                               , amadeus
                                                                               , 100 ).await;
                                          if succ {
                                            track.bets.push(bet);
                                          } else {
                                            error!("Error on bet {:?}", rst);
                                          }
                                        }
                                      }
                                    }
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    } else {
                      let _ = msg_id.delete_reactions(&xtx_clone.http).await;
                      break;
                    }
                  }
                });
              },
              Err(why) => {
                error!("Failed to post live match {why}");
                error!("Fields: {:?}\n{:?}\n{:?}\n", game.description, image, em_url);
              }
            }

            } // if there is specific game channel
            } // if there are discord fields

          }
        }
      }

      set!{ nao             = chrono::offset::Utc::now()
          , current_weekday = nao.weekday() };
      if current_weekday == Weekday::Sun {
        checker::CURRENT_TIMEOUT.store(NIGHT_TIMEOUT_SECS, Ordering::Relaxed);
        tokio::time::sleep(NIGHT_TIMEOUT).await;
      } else {
        let current_hour_utc = nao.hour();
        if current_hour_utc > 2 && current_hour_utc < 15 {
          checker::CURRENT_TIMEOUT.store(DAY_TIMEOUT_SECS, Ordering::Relaxed);
          tokio::time::sleep(DAY_TIMEOUT).await;
        } else {
          checker::CURRENT_TIMEOUT.store(NIGHT_TIMEOUT_SECS, Ordering::Relaxed);
          tokio::time::sleep(NIGHT_TIMEOUT).await;
        }
      }
    }
  });
}

use crate::{
  types::{ common::ReqwestClient
         , options::IOptions
         , tracking::TrackingGame
         , tracking::Bet
         , twitch::Twitch
         , goodgame::GoodGameData },
  common::trees,
  steins::cyber
};

use serenity::{
  prelude::*,
  model::{ id::ChannelId
         , channel::ReactionType }
};

use std::{ time
         , sync::Arc };

use rand::Rng;

pub async fn activate_games_tracking(
                     ctx:       &Arc<Context>
                   , options:   &IOptions
                   , token:     String
                   , amadeus:   u64 ) {

  set!{ ch_deref      = ChannelId( 721956117558853673 )
      , ctx_clone     = Arc::clone(&ctx)
      , options_clone = options.clone() };

  // Delete live games from log channel (if some)
  if let Ok(vec_msg) = ch_deref.messages(&ctx, |g| g.limit(50)).await {
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
      match ch_deref.delete_messages(&ctx, vec_id.as_slice()).await {
        Ok(nothing)  => nothing,
        Err(err) => warn!("Failed to clean live messages {}", err),
      };
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
        let mut games_lock = cyber::team_checker::GAMES.lock().await;
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
          games_lock.remove(&ktd);
        }
      }

      info!("check");
      let our_gsx = cyber::team_checker::check( &ctx_clone
                                              , ch_deref.0
                                              , options_clone.guild
                                              , &rqcl
                                              ).await;

      for game in our_gsx {
        let game_key = game.key.clone();
        let playa = &game.players[0];
        if let Ok(user) = ctx_clone.http.get_user(playa.discord).await {

          setm!{ twitch_live        = false
                , additional_fields  = Vec::new()
                , image              = None
                , em_url             = None };

          if playa.streams.is_some() {
            let streams = playa.streams.clone().unwrap();
            if streams.twitch.is_some() {
              let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", &streams.twitch.unwrap());
              if let Ok(res) = rqcl
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
                        let titurl = format!("{}\n{}", &twd.title, url);
                        additional_fields.push(("Live on twitch", titurl, false));
                        image       = Some(pic);
                        em_url      = Some(url);
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
              let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{}", &ggru);
              if let Ok(gg) = rqcl.get(&ggru_link).send().await {
                match gg.json::<GoodGameData>().await {
                  Ok(ggdata) => {
                    if ggdata.status == "Live" {
                      let url = format!("https://goodgame.ru/channel/{}", &ggru);
                      if twitch_live {
                        let titurl =
                          format!("{}\n{}", &ggdata.channel.title, url);
                        additional_fields.push(("Live on ggru", titurl, true));
                      } else {
                        additional_fields.push(("Live on ggru", ggdata.channel.title.clone(), false));
                        image  = Some(ggdata.channel.thumb.clone());
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
          set!{ red   = rand::thread_rng().gen_range(0, 255)
              , green = rand::thread_rng().gen_range(0, 255)
              , blue  = rand::thread_rng().gen_range(0, 255) };

          let nickname_maybe = user.nick_in(&ctx_clone.http, options_clone.guild).await;
          let nick = nickname_maybe.unwrap_or_else(|| user.name.clone());

          match ch_deref.send_message(&ctx_clone, |m| m
            .embed(|e| {
              let mut e = e
                .title("JUST STARTED")
                .author(|a| a.icon_url(&user.face()).name(&nick))
                .colour((red, green, blue));
              if !game.description.is_empty() {
                e = e.description(&game.description[0]);
                if game.description.len() > 2 {
                  let d_fields = vec![
                    ("Team 1", &game.description[1], true)
                  , ("Team 2", &game.description[2], true)
                  ];
                  e = e.fields(d_fields);
                }
              }
              if !additional_fields.is_empty() {
                e = e.fields(additional_fields);
              }
              if let Some(some_image) = image {
                e = e.image(some_image);
              }
              if let Some(some_url) = em_url {
                e = e.url(some_url);
              }
              e
            }
          )).await {
            Ok(msg_id) => {
              { // scope for games_lock
                let mut games_lock = cyber::team_checker::GAMES.lock().await;
                games_lock.insert(game_key.clone(), TrackingGame {
                  tracking_msg_id: msg_id.id.0,
                  passed_time: 0,
                  still_live: false,
                  players: game.players, bets: vec![], fails: 0 }
                );
              }
              let up = ReactionType::Unicode(String::from("👍🏻"));
              let dw = ReactionType::Unicode(String::from("👎🏻"));
              let _ = msg_id.react(&ctx_clone.http, up).await;
              let _ = msg_id.react(&ctx_clone.http, dw).await;
              // run thread inside thread for reactions
              // we're cloning ctx yet another time here!
              let xtx_clone = Arc::clone(&ctx_clone);
              tokio::spawn(async move {
                loop {
                  // 20 minutes for each game
                  if let Some(reaction) =
                    &msg_id.await_reaction(&xtx_clone.shard)
                            .timeout(time::Duration::from_secs(1200)).await {
                    let inref = reaction.as_inner_ref();
                    let emoji = &inref.emoji;
                    if let Some(u) = inref.user_id {
                      if let Some(g) = inref.guild_id {
                        if let Ok(p) = trees::get_points( g.0, u.0 ).await {
                          if p > 100 {
                            let emoji_data = emoji.as_data();
                            if emoji_data.as_str() == "👍🏻" || emoji_data.as_str() == "👎🏻" {
                              let is_positive = emoji_data.as_str() == "👍🏻";
                              let mut gl = cyber::team_checker::GAMES.lock().await;
                              if let Some(track) = gl.get_mut(&game_key) {
                                if track.still_live {
                                  // you bet only once
                                  if !track.bets.iter().any(|b| b.member == u.0) {
                                    let bet = Bet { guild: g.0
                                                  , member: u.0
                                                  , points: 100
                                                  , positive: is_positive };
                                    let (succ, rst) = trees::give_points( g.0, u.0
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
                  } else {
                    let _ = msg_id.delete_reactions(&xtx_clone.http).await;
                    break;
                  }
                }
              });
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

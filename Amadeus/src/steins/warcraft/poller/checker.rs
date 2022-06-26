use crate::{
  types::{ serenity::{ CoreGuild, CoreGuilds }
         , team::DiscordPlayer
         , tracking::*
         , w3c::Going },
  collections::team::{ PLAYERS, DISCORDS },
  common::{ constants::W3C_API
          , help::fields::{ FieldsVec, FieldsVec2 }
          , db::trees::points },
  steins::warcraft::{
    aka_checker::aka,
    poller::{ GAMES
            , finished::check_match
            , bet_fields::generate_bet_fields },
    utils::{ get_race2
           , get_map },
    status::{
      add_to_weekly,
      status_update
    },
    flotv::get_flotv
  }
};

use serenity::{
  prelude::*,
  model::id::{ UserId
             , GuildId
             , ChannelId }
};

use std::collections::HashMap;

#[allow(clippy::needless_range_loop)]
pub async fn check<'a>( ctx: &Context
                      , guild_id: u64
                      , rqcl: &reqwest::Client
                      ) -> Vec<StartingGame<'a>> {

  let guild = GuildId( to_nzu!(guild_id) );
  let mut out: Vec<StartingGame> = Vec::new();
  let mut stats: W3CStats = W3CStats { ..Default::default() };

  if let Ok(res) =
    rqcl.get(&format!("{W3C_API}/matches/ongoing?offset=0&gameMode=1"))
        .send()
        .await {
    trace!("team games: checking solo matches");
    if let Ok(going) = res.json::<Going>().await {
      let games_solo = going.matches.len();
      trace!("team games: {} matches", games_solo);
      stats.games_solo = games_solo;
      if !going.matches.is_empty() {
        for m in going.matches {
          if m.gameMode == 1 { // solo
            if m.teams.len() > 1 && !m.teams[0].players.is_empty() && !m.teams[1].players.is_empty() {
              let playaz = PLAYERS.iter().copied().filter( |p|
                   m.teams[0].players[0].battleTag == p.player.battletag
                || m.teams[1].players[0].battleTag == p.player.battletag
                || if !p.player.alt_accounts.is_empty() {
                  p.player.alt_accounts.iter().any(|alt_accounts|
                     &m.teams[0].players[0].battleTag == alt_accounts
                  || &m.teams[1].players[0].battleTag == alt_accounts
                  )
                } else { false }).collect::<Vec<&DiscordPlayer>>();
              if !playaz.is_empty() {
                set!{ g_map   = get_map(&m.map)
                    , race1   = get_race2(m.teams[0].players[0].race)
                    , race2   = get_race2(m.teams[1].players[0].race)
                    , t0_name = aka(&m.teams[0].players[0], rqcl).await
                    , t1_name = aka(&m.teams[1].players[0], rqcl).await };

                setm!{ t0_ping = String::new()
                     , t1_ping = String::new() };

                let host = m.serverInfo.name.unwrap_or_else(|| "no information about host".into());
                if m.serverInfo.playerServerInfos.len() > 1 {
                  let (t0_index, t1_index) =
                    if m.teams[0].players[0].battleTag == m.serverInfo.playerServerInfos[0].battleTag {
                      (0, 1)
                    } else {
                      (1, 0)
                    };
                  t0_ping = format!("\n*avg: {}ms now: {}ms*", m.serverInfo.playerServerInfos[t0_index].averagePing
                                                             , m.serverInfo.playerServerInfos[t0_index].currentPing);
                  t1_ping = format!("\n*avg: {}ms now: {}ms*", m.serverInfo.playerServerInfos[t1_index].averagePing
                                                             , m.serverInfo.playerServerInfos[t1_index].currentPing);
                }

                let mvec =
                  vec![ format!("Map: **{g_map}**")
                      , format!("({race1}) **{t0_name}** [{}]{}", m.teams[0].players[0].oldMmr, t0_ping)
                      , format!("({race2}) **{t1_name}** [{}]{}", m.teams[1].players[0].oldMmr, t1_ping) ];

                trace!("team games: locking solo");
                { // games lock scope
                  let mut games_lock = GAMES.lock().await;
                  if let Some(track) = games_lock.get_mut(&m.match_id) {
                    track.still_live = true;
                    set!{ minutes     = track.passed_time // team_games.rs poll time
                        , footer      = format!("Passed: {minutes} min")
                        , playa       = playaz[0].player.discord
                        , bet_fields  = generate_bet_fields(ctx, track).await };
                    for t in track.tracking_msg_id.iter() {
                    if let Some(ds) = DISCORDS.get(&t.0) {
                    if let Some(ch) = ds.games {

                    if let Ok(mut msg) = ctx.http.get_message(ch, t.1).await {
                      if let Ok(user) = ctx.http.get_user(playa).await {

                        if track.flo_tv.is_none() {
                          if let Ok(Some(flotv)) = get_flotv(rqcl, &playaz).await {
                            track.flo_tv = Some(flotv);
                          }
                        };

                        setm!{ fields = Vec::new()
                             , img    = None
                             , url    = None
                             , color  = None };
                        if !msg.embeds.is_empty() {
                          if !msg.embeds[0].fields.is_empty() {
                            for f in msg.embeds[0].fields.clone() {
                              if f.name != "Bets"
                              && f.name != "flo tv" {
                                fields.push((f.name, f.value, f.inline));
                              }
                            }
                          }
                          img   = msg.embeds[0].image.clone();
                          url   = msg.embeds[0].url.clone();
                          color = msg.embeds[0].colour;
                        };

                        let nick = user.nick_in(&ctx.http, guild)
                                       .await.unwrap_or_else(|| user.name.clone());

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e|  {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&nick))
                              .description(&mvec[0])
                              .footer(|f| f.text(&footer));
                            if !fields.is_empty() {
                              e = e.fields_vec(fields);
                            }
                            if let Some(bet_data) = &bet_fields {
                              e = e.fields_vec(bet_data.clone());
                            }
                            if let Some(some_img) = img {
                              e = e.image(&some_img.url);
                            }
                            if let Some(some_url) = url {
                              e = e.url(some_url);
                            }
                            if let Some(colour) = color {
                              e = e.colour(colour);
                            }
                            if let Some(flotv) = &track.flo_tv {
                              e = e.fields([("flo tv", flotv.as_str(), false)]);
                            }
                            e
                          }
                        )).await {
                          error!("Failed to post live match {why}");
                        }
                      }
                    }
                    } // if channel found
                    } // if discord found
                    } // for all tracking messages

                  } else {
                    out.push(
                      StartingGame { key: m.match_id
                                   , description: mvec
                                   , players: playaz
                                   , host
                                   , mode: GameMode::Solo });
                  }
                }
              }
            }
          }
        }
      }
    } else {
      warn!("team games: backend gives no solo games, blame w3c");
    }
  }

  if let Ok(res) =
    rqcl.get(&format!("{W3C_API}/matches/ongoing?offset=0&gameMode=2"))
        .send()
        .await {
    trace!("team games: checking 2x2 matches");
    if let Ok(going) = res.json::<Going>().await {
      let games_2x2 = going.matches.len();
      trace!("team games: {} matches", games_2x2);
      stats.games_2x2 = games_2x2;
      if !going.matches.is_empty() {
        for m in going.matches {
           if m.gameMode == 6 || m.gameMode == 2 { // AT or RT mode 2x2
            if m.teams.len() > 1 && m.teams[0].players.len() > 1 && m.teams[1].players.len() > 1 {
              let playaz = PLAYERS.iter().copied().filter( |p|
                   m.teams[0].players[0].battleTag == p.player.battletag
                || m.teams[1].players[0].battleTag == p.player.battletag
                || m.teams[0].players[1].battleTag == p.player.battletag
                || m.teams[1].players[1].battleTag == p.player.battletag
                || if !p.player.alt_accounts.is_empty() {
                  p.player.alt_accounts.iter().any(|other_acc|
                     &m.teams[0].players[0].battleTag == other_acc
                  || &m.teams[1].players[0].battleTag == other_acc
                  || &m.teams[0].players[1].battleTag == other_acc
                  || &m.teams[1].players[1].battleTag == other_acc
                  )
              } else { false }).collect::<Vec<&DiscordPlayer>>();

              if !playaz.is_empty() {
                let g_map = get_map(&m.map);

                set! { race1  = get_race2(m.teams[0].players[0].race)
                     , race12 = get_race2(m.teams[0].players[1].race)
                     , race2  = get_race2(m.teams[1].players[0].race)
                     , race22 = get_race2(m.teams[1].players[1].race) };

                let mut aka_names: [[String; 2]; 2] = Default::default();
                for i in 0..2 {
                  for j in 0..2 {
                    aka_names[i][j] = aka(&m.teams[i].players[j], rqcl).await;
                  }
                }

                let mvec =
                  if m.gameMode == 6 {
                    let team1 = format!("({}) **{}**\n({}) **{}**\n{} MMR"
                      , race1, aka_names[0][0]
                      , race12, aka_names[0][1], m.teams[0].players[0].oldMmr);
                    let team2 = format!("({}) **{}**\n({}) **{}**\n{} MMR"
                      , race2, aka_names[1][0]
                      , race22, aka_names[1][1], m.teams[1].players[0].oldMmr);
                    vec![ format!("Map: **{g_map}**"), team1, team2 ]
                  } else {
                    let team1 = format!("({}) **{}** [{}]\n({}) **{}** [{}]"
                      , race1, aka_names[0][0], m.teams[0].players[0].oldMmr
                      , race12, aka_names[0][1], m.teams[0].players[1].oldMmr);
                    let team2 = format!("({}) **{}** [{}]\n({}) **{}** [{}]"
                      , race2, aka_names[1][0], m.teams[1].players[0].oldMmr
                      , race22, aka_names[1][1], m.teams[1].players[1].oldMmr);
                    vec![ format!("Map: **{g_map}**"), team1, team2 ]
                  };

                let host = m.serverInfo.name.unwrap_or_else(|| "no information about host".into());

                trace!("team games: locking 2x2");
                { // games lock scope
                  let mut games_lock = GAMES.lock().await;
                  if let Some(track) = games_lock.get_mut(&m.match_id) {
                    track.still_live = true;
                    set!{ minutes = track.passed_time // team_games.rs poll time
                        , footer  = format!("Passed: {minutes} min") };

                    let bet_fields = generate_bet_fields(ctx, track).await;
                    for t in track.tracking_msg_id.iter() {
                    if let Some(ds) = DISCORDS.get(&t.0) {
                    if let Some(ch) = ds.games2 {

                    if let Ok(mut msg) = ctx.http.get_message(ch, t.1).await {
                      // get first player for discord
                      let playa = playaz[0].player.discord;
                      if let Ok(user) = ctx.http.get_user(playa).await {

                        if track.flo_tv.is_none() {
                          if let Ok(Some(flotv)) = get_flotv(rqcl, &playaz).await {
                            track.flo_tv = Some(flotv);
                          }
                        };

                        setm!{ fields = Vec::new()
                             , img    = None
                             , url    = None
                             , color  = None };
                        if !msg.embeds.is_empty() {
                          if !msg.embeds[0].fields.is_empty() {
                            for f in msg.embeds[0].fields.clone() {
                              if f.name != "Bets"
                              && f.name != "flo tv" {
                                fields.push((f.name, f.value, f.inline));
                              }
                            }
                          }
                          img   = msg.embeds[0].image.clone();
                          url   = msg.embeds[0].url.clone();
                          color = msg.embeds[0].colour;
                        };

                        let nick = user.nick_in(&ctx.http, guild)
                                       .await.unwrap_or_else(|| user.name.clone());

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e| {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&nick))
                              .description(&mvec[0])
                              .footer(|f| f.text(&footer));
                            if !fields.is_empty() {
                              e = e.fields_vec(fields);
                            }
                            if let Some(bet_data) = &bet_fields {
                              e = e.fields_vec(bet_data.clone());
                            }
                            if let Some(some_img) = img {
                              e = e.image(&some_img.url);
                            }
                            if let Some(some_url) = url {
                              e = e.url(some_url);
                            }
                            if let Some(colour) = color {
                              e = e.colour(colour);
                            }
                            if let Some(flotv) = &track.flo_tv {
                              e = e.fields([("flo tv", flotv.as_str(), false)]);
                            }
                            e
                          }
                        )).await {
                          error!("Failed to post live match {why}");
                        }
                      }
                    }

                    } // if channel found
                    } // if discord found
                    } // for all trackign messages

                  } else {
                    out.push(
                      StartingGame { key: m.match_id
                                   , description: mvec
                                   , players: playaz
                                   , host
                                   , mode: GameMode::Team2 });
                  }
                }
              }
            }
          }
        }
      }
    } else {
      warn!("team games: backend gives no 2x2 games, blame w3c");
    }
  }

  if let Ok(res) =
    rqcl.get(&format!("{W3C_API}/matches/ongoing?offset=0&gameMode=4"))
        .send()
        .await {
    trace!("team games: checking 4x4 matches");
    if let Ok(going) = res.json::<Going>().await {
      let games_4x4 = going.matches.len();
      trace!("team games: {} matches", games_4x4);
      stats.games_4x4 = games_4x4;
      if !going.matches.is_empty() {
        for m in going.matches {
          if m.gameMode == 4 && // 4x4
            m.teams.len() > 1 && m.teams[0].players.len() > 3 && m.teams[1].players.len() > 3 {
            let playaz = PLAYERS.iter().copied().filter( |p|
                 m.teams[0].players[0].battleTag == p.player.battletag || m.teams[0].players[2].battleTag == p.player.battletag
              || m.teams[1].players[0].battleTag == p.player.battletag || m.teams[1].players[2].battleTag == p.player.battletag
              || m.teams[0].players[1].battleTag == p.player.battletag || m.teams[0].players[3].battleTag == p.player.battletag
              || m.teams[1].players[1].battleTag == p.player.battletag || m.teams[1].players[3].battleTag == p.player.battletag
              || if !p.player.alt_accounts.is_empty() {
                p.player.alt_accounts.iter().any(|other_acc|
                  &m.teams[0].players[0].battleTag == other_acc || &m.teams[0].players[2].battleTag == other_acc
                || &m.teams[1].players[0].battleTag == other_acc || &m.teams[1].players[2].battleTag == other_acc
                || &m.teams[0].players[1].battleTag == other_acc || &m.teams[0].players[3].battleTag == other_acc
                || &m.teams[1].players[1].battleTag == other_acc || &m.teams[1].players[3].battleTag == other_acc
                )
            } else { false }).collect::<Vec<&DiscordPlayer>>();

            if !playaz.is_empty() {
              let g_map = get_map(&m.map);

              set!{ race1  = get_race2(m.teams[0].players[0].race), race13 = get_race2(m.teams[0].players[2].race)
                  , race12 = get_race2(m.teams[0].players[1].race), race14 = get_race2(m.teams[0].players[3].race)
                  , race2  = get_race2(m.teams[1].players[0].race), race23 = get_race2(m.teams[1].players[2].race)
                  , race22 = get_race2(m.teams[1].players[1].race), race24 = get_race2(m.teams[1].players[3].race) };

              let mut aka_names: [[String; 4]; 2] = Default::default();
              for i in 0..2 {
                for j in 0..4 {
                  aka_names[i][j] = aka(&m.teams[i].players[j], rqcl).await;
                }
              }

              let mvec = {
                  let team1 = format!("({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]"
                    , race1,  aka_names[0][0], m.teams[0].players[0].oldMmr
                    , race12, aka_names[0][1], m.teams[0].players[1].oldMmr
                    , race13, aka_names[0][2], m.teams[0].players[2].oldMmr
                    , race14, aka_names[0][3], m.teams[0].players[3].oldMmr);
                  let team2 = format!("({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]"
                    , race2,  aka_names[1][0], m.teams[1].players[0].oldMmr
                    , race22, aka_names[1][1], m.teams[1].players[1].oldMmr
                    , race23, aka_names[1][2], m.teams[1].players[2].oldMmr
                    , race24, aka_names[1][3], m.teams[1].players[3].oldMmr);
                  vec![ format!("Map: **{g_map}**"), team1, team2 ]
                };

              let host = m.serverInfo.name.unwrap_or_else(|| "no information about host".into());

              trace!("team games: locking 4x4");
              { // games lock scope
                let mut games_lock = GAMES.lock().await;
                if let Some(track) = games_lock.get_mut(&m.match_id) {
                  track.still_live = true;
                  set!{ minutes = track.passed_time
                      , footer = format!("Passed: {minutes} min") };

                  let bet_fields = generate_bet_fields(ctx, track).await;
                  for t in track.tracking_msg_id.iter() {
                  if let Some(ds) = DISCORDS.get(&t.0) {
                  if let Some(ch) = ds.games4 {

                  if let Ok(mut msg) = ctx.http.get_message(ch, t.1).await {
                    // get first player for discord
                    let playa = playaz[0].player.discord;
                    if let Ok(user) = ctx.http.get_user(playa).await {
                      setm!{ fields = Vec::new()
                           , img    = None
                           , url    = None
                           , color  = None };
                      if !msg.embeds.is_empty() {
                        if !msg.embeds[0].fields.is_empty() {
                          for f in msg.embeds[0].fields.clone() {
                            if f.name != "Bets"
                            && f.name != "flo tv" {
                              fields.push((f.name, f.value, f.inline));
                            }
                          }
                        }
                        img   = msg.embeds[0].image.clone();
                        url   = msg.embeds[0].url.clone();
                        color = msg.embeds[0].colour;
                      };

                      let nick = user.nick_in(&ctx.http, guild)
                                     .await.unwrap_or_else(|| user.name.clone());

                      if let Err(why) = msg.edit(ctx, |m| m
                        .embed(|e| {
                          let mut e = e
                            .title("LIVE")
                            .author(|a| a.icon_url(&user.face()).name(&nick))
                            .description(&mvec[0])
                            .footer(|f| f.text(&footer));
                          if !fields.is_empty() {
                            e = e.fields_vec(fields);
                          }
                          if let Some(bet_data) = &bet_fields {
                            e = e.fields_vec(bet_data.clone());
                          }
                          if let Some(some_img) = img {
                            e = e.image(&some_img.url);
                          }
                          if let Some(some_url) = url {
                            e = e.url(some_url);
                          }
                          if let Some(colour) = color {
                            e = e.colour(colour);
                          }
                          e
                        }
                      )).await {
                        error!("Failed to post live match {why}");
                      }
                    }
                  }

                  } // if channel found
                  } // if discord found
                  } // for all tracking

                } else {
                  out.push(
                    StartingGame { key: m.match_id
                                 , description: mvec
                                 , players: playaz
                                 , host
                                 , mode: GameMode::Team4 });
                }
              }
            }
          }
        }
      }
    } else {
      warn!("team games: backend gives no 4x4 games, blame w3c");
    }
  }

  // TODO: too big logic for errors checking!
  if let Err(what) = status_update(ctx, &stats).await {
    if !what.to_string().contains("connection closed before message completed") {
      if let Ok(res_test) = rqcl.get("https://matchmaking-service.w3champions.com/queue/snapshots").send().await {
        if let Ok(text_res) = res_test.text().await {
          error!("Failed to update W3C status: {what} on {text_res}");
        } else {
          error!("Failed to update W3C status and text: {what}");
        }
      } else {
        error!("Failed to update W3C status, no answer from server: {what}");
      }
    } else {
      error!("Failed to update W3C status: {what}");
    }
  }

  { // games lock scope
    trace!("team games: finishing checking");
    let mut k_to_del: Vec<String> = Vec::new();
    let mut games_lock = GAMES.lock().await;
    for (k, track) in games_lock.iter_mut() {
      if !track.still_live {

        // get first player for (again, as ususal)
        let playa = track.players[0].clone();

        for d in playa.discords.iter() {
        if let Some(ds) = DISCORDS.get(d) {

        let game_channel_maybe = match track.mode {
          GameMode::Solo  => ds.games,
          GameMode::Team2 => ds.games2,
          GameMode::Team4 => ds.games4
        };

        if let Some(gc) = game_channel_maybe {
        let game_channel = ChannelId(to_nzu!(gc));

        if let Some(finished_game) = check_match(k, &track.players, &track.mode, rqcl).await {
          let fgame = &finished_game;
          if let Ok(mut msg) = ctx.http.get_message( game_channel.0.get()
                                                   , track.tracking_msg_id[0].1 ).await {
            let footer: String = format!("Passed: {} min", fgame.passed_time);
            if let Ok(user) = ctx.http.get_user(playa.player.discord).await {
              let mut old_fields = Vec::new();
              let mut color = None;
              if !msg.embeds.is_empty() {
                if !msg.embeds[0].fields.is_empty() {
                  for f in msg.embeds[0].fields.clone() {
                    if f.name != "Team 1"
                    && f.name != "Team 2"
                    && f.name != "Bets" {
                      old_fields.push((f.name, f.value, f.inline));
                    }
                  }
                }
                color = msg.embeds[0].colour;
              };
              setm!{ title          = "FINISHED"
                   , streak_fields  = None
                   , bet_fields     = None };
              for ((pws, pw), is_win) in &fgame.winners {
                let is_solo = match track.mode {
                  GameMode::Solo  => true,
                  GameMode::Team2 => false,
                  GameMode::Team4 => false
                };
                if let Err(why) = add_to_weekly(ctx, pws, *is_win, is_solo).await {
                  error!("Failed to add stats: {why}");
                }
                if *is_win {
                  trace!("Registering win for {pw}");
                  let streak = points::add_win_points( guild_id
                                                     , *pw
                                                     ).await;
                  if playa.player.discord == *pw && streak >= 3 {
                    title =
                      match streak { 3  => "MULTIKILL"
                                   , 4  => "MEGA KILL"
                                   , 5  => "ULTRAKILL"
                                   , 6  => "KILLING SPREE"
                                   , 7  => "RAMPAGE!"
                                   , 8  => "DOMINATING"
                                   , 9  => "UNSTOPPABLE"
                                   , 10 => "GODLIKE!"
                                   , 11 => "WICKED SICK"
                                   , 12 => "ALPHA"
                                   , _  => "FRENETIC" };
                    let dd = format!("Doing _**{streak}**_ kills in a row**!**");
                    streak_fields = Some(vec![("Winning streak", dd, false)]);
                  }
                } else {
                  trace!("Registering lose for {pw}");
                  points::break_streak(guild_id, *pw).await;
                }
                if !track.bets.is_empty() && bet_fields.is_none() {
                  trace!("Paying for bets");
                  let amadeus_maybe = {
                    let data = ctx.data.read().await;
                    if let Some(core_guilds) = data.get::<CoreGuilds>() {
                      core_guilds.get(&CoreGuild::Amadeus).copied()
                    } else { None }
                  };
                  // There is complicated bet win calculation
                  if let Some(amadeus) = amadeus_maybe {
                    if let Ok(p) = points::get_points( guild_id, amadeus ).await {
                      setm!{ win_calculation  = HashMap::new()
                           , waste            = 0
                           , k                = 2.0f32
                           , losers_output    = vec![] };
                      for bet in &track.bets {
                        if *is_win == bet.positive {
                          let best_win = 
                            if bet.registered {
                              (bet.points as f32 * k).round() as u64
                            } else {
                              bet.points
                            };
                          win_calculation.insert(bet.member, (bet.points, best_win));
                          waste += best_win;
                        } else {
                          let user_id = UserId( to_nzu!(bet.member) );
                          if let Ok(user) = user_id.to_user(&ctx).await {
                            losers_output.push(
                              format!("**{}** loses **{}**", user.name, bet.points)
                            );
                          }
                        }
                      }
                      while waste > p {
                        k -= 0.1;
                        waste = 0;
                        for (_, (_, wpp)) in win_calculation.iter_mut() {
                          *wpp = (*wpp as f32 * k).round() as u64;
                          waste += *wpp;
                        }
                      }
                      let mut output = vec![];
                      for (mpp, (ppp, wpp)) in win_calculation.iter() {
                        let (succ, rst) =
                          points::give_points( guild_id
                                             , amadeus
                                             , *mpp
                                             , *wpp ).await;
                        if !succ {
                          error!("failed to give bet win points: {rst}");
                        } else {
                          let user_id = UserId( to_nzu!(*mpp) );
                          if let Ok(user) = user_id.to_user(&ctx).await {
                            let pure_win = *wpp - *ppp;
                            output.push(
                              format!("**{}** wins **{pure_win}**", user.name)
                            );
                          }
                        }
                      }
                      let title = format!("Bets coefficient: {k}");
                      if !output.is_empty() || !losers_output.is_empty() {
                        let mut out_fields = vec![];
                        if !output.is_empty() {
                          out_fields.push(
                            (title, output.join("\n")
                                  , false)
                          );
                        }
                        if !losers_output.is_empty() {
                          out_fields.push(
                            ("Betting losers".to_string()
                                  , losers_output.join("\n")
                                  , false)
                          );
                        }
                        bet_fields = Some(out_fields);
                      }
                    }
                  }
                }
              }

              let nick = user.nick_in(&ctx.http, guild)
                             .await.unwrap_or_else(|| user.name.clone());

              if let Err(why) = msg.edit(ctx, |m| m
                .embed(|e| {
                  let mut e =
                    e.author(|a| a.icon_url(&user.face()).name(&nick))
                     .title(title)
                     .url(&fgame.link)
                     .footer(|f| f.text(footer));
                  if !fgame.desc.is_empty() {
                    e = e.description(&fgame.desc[0]);
                    if fgame.desc.len() > 2 {
                      let d_fields = vec![
                        ("Team 1", fgame.desc[1].as_str(), true)
                      , ("Team 2", fgame.desc[2].as_str(), true)
                      ];
                      e = e.fields(d_fields);
                    } else {
                      // TODO: drop it, this should never happen
                      e = e.description(&fgame.desc[0]);
                    }
                  }
                  if !old_fields.is_empty() {
                    e = e.fields_vec(old_fields);
                  }
                  if let Some(streak_data) = streak_fields {
                    e = e.fields_vec2(streak_data);
                  }
                  if let Some(bet_data) = bet_fields {
                    e = e.fields_vec(bet_data);
                  }
                  if let Some((s1,s2,s3,s4)) = &fgame.additional_fields {
                    e = e.fields(vec![
                      (s1.as_str(), s3.as_str(), true),
                      (s2.as_str(), s4.as_str(), true)
                    ]);
                  }
                  if let Some(hero) = &fgame.hero_png {
                    e.thumbnail(hero);
                  }
                  if let Some(colour) = color {
                    e.colour(colour);
                  }
                  e
                })
              ).await {
                error!("Failed to update live match {:?}", why);
              }
            }
          }
          // we only delete match if it's passed
          // if not possibly there is a bug and we're waiting for end
          k_to_del.push(k.clone());
        } else {
          // game is not finished but not live
          if track.fails < 3 {
            track.fails += 1;
          } else {
            // mark tracking game for removal after 3 fails
            k_to_del.push(k.clone());
            if let Ok(msg) = ctx.http.get_message( game_channel.0.get()
                                                 , track.tracking_msg_id[0].1 ).await {
              if let Err(wtf) = msg.delete(ctx).await {
                error!("Failed to clean up dropped Live game {:?}", wtf);
              }
            }
          }
        }

        } // find channel for game mode
        } // find discord
        } // iterate discords

      }
    }

    for ktd in k_to_del {
      games_lock.remove(&ktd);
    }
  } // games lock scope end

  out
}

use crate::{
  types::{
    tracking::*,
    w3c::{ Going, MD }
  },
  collections::team::teammates,
  common::points,
  stains::cyber::{
    utils::{ get_race2, get_map }
  }
};

use serenity::{
  prelude::*
};

use std::collections::HashMap;
use tokio::sync::{ Mutex, MutexGuard };

lazy_static! {
  pub static ref GAMES: Mutex<HashMap<String, TrackingGame>>
    = Mutex::new(HashMap::new());
}

async fn check_match( matchid_lol: &str
                    , btag: &str
                    ) -> Option<FinishedGame> {
  let mut are_you_winning = false;
  let mut matchid_s : String = String::new();
  if let Ok(wtf) = reqwest::get("https://statistic-service.w3champions.com/api/matches?offset=0&gateway=20").await {
    if let Ok(going) = wtf.json::<Going>().await {
      for mm in &going.matches {
        if mm.startTime == matchid_lol {
          // TODO: change that hack one day
          if
            if mm.gameMode == 6 || mm.gameMode == 2 {
              mm.teams[0].players[0].battleTag == btag || mm.teams[1].players[0].battleTag == btag ||
              mm.teams[0].players[1].battleTag == btag || mm.teams[1].players[1].battleTag == btag
            } else {
              mm.teams[0].players[0].battleTag == btag || mm.teams[1].players[0].battleTag == btag
            }
          {
            matchid_s = mm.id.clone();
            break;
          }
        }
      }
    }
  }

  if matchid_s.is_empty() { return None; }
  let matchid = matchid_s.as_str();
  let url = format!("https://statistic-service.w3champions.com/api/matches/{}", matchid);

  if let Ok(res) = reqwest::get(url.as_str()).await {
    match res.json::<MD>().await {
      Ok(md) => {
        let m = md.match_data;
        let mstr_o =
        if m.gameMode == 1 {
          set!{ g_map = get_map(m.map.as_str())
              , race1 = get_race2(m.teams[0].players[0].race)
              , race2 = get_race2(m.teams[1].players[0].race) };
          let player1 = if m.teams[0].players[0].won {
            if m.teams[0].players[0].battleTag == btag {
              are_you_winning = true;
            }
            format!("__**{}**__ **+{}**", m.teams[0].players[0].name, m.teams[0].players[0].mmrGain)
          } else {
            format!("__*{}*__ **{}**", m.teams[0].players[0].name, m.teams[1].players[0].mmrGain)
          };
          let player2 = if m.teams[1].players[0].won {
            format!("__**{}**__ **+{}**", m.teams[1].players[0].name, m.teams[0].players[0].mmrGain)
          } else {
            format!("__*{}*__ **{}**", m.teams[1].players[0].name, m.teams[1].players[0].mmrGain)
          };
          Some(format!("({}) {} [{}] *vs* ({}) {} [{}] *{}*",
              race1, player1, m.teams[0].players[0].oldMmr
            , race2, player2, m.teams[1].players[0].oldMmr, g_map))
        } else if m.gameMode == 6 || m.gameMode == 2 {
          set!{ g_map  = get_map(m.map.as_str())
              , race1  = get_race2(m.teams[0].players[0].race)
              , race12 = get_race2(m.teams[0].players[1].race)
              , race2  = get_race2(m.teams[1].players[0].race)
              , race22 = get_race2(m.teams[1].players[1].race) };
          if m.teams[0].won
          && ( m.teams[0].players[0].battleTag == btag
            || m.teams[0].players[1].battleTag == btag ) {
            are_you_winning = true;
          }
          if m.gameMode == 6 {
            if m.teams[0].won {
              Some(format!("({}+{}) __**{} + {} [{}]**__ **+{}** (won)\n*vs*\n({}+{}) __*{} + {} [{}]*__ *{}* (lost)\n\nmap: **{}**",
                race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, g_map))
            } else {
              Some(format!("({}+{}) __*{} + {} [{}]*__ *{}* (lost)\n*vs*\n({}+{}) __**{} + {} [{}]**__ **+{}** (won)\n\nmap: **{}**",
                race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain
              , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, g_map))
            }
          } else if m.teams[0].won {
            Some(format!("({}+{}) __**{} [{}]**__ **+{}** + __**{} [{}]**__ **+{}** (won)\n*vs*\n({}+{}) __*{} [{}]*__ *{}* + __*{} [{}]*__ *{}* (lost)\n\nmap: **{}**",
              race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr, m.teams[0].players[1].mmrGain
            , race2, race22, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, m.teams[1].players[1].mmrGain, g_map))
          } else {
            Some(format!("({}+{}) __*{} [{}]*__ *{}* + __*{} [{}]*__ *{}* (lost)\n*vs*\n({}+{}) __**{} [{}]**__ **+{}** + __**{} [{}]**__ **+{}** (won)\n\nmap: **{}**",
            race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[0].mmrGain, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr, m.teams[0].players[1].mmrGain
            , race2, race22, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, m.teams[1].players[0].mmrGain, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, m.teams[1].players[1].mmrGain, g_map))
          }
        } else {
          None
        };
        match mstr_o {
          Some(mstr) => {
            let duration_in_minutes = m.durationInSeconds / 60;
            if md.playerScores.len() > 1 && m.gameMode == 1 {
              set! { p1 = &md.playerScores[0]
                   , p2 = &md.playerScores[1]
                   , s1 = p1.battleTag.clone()
                   , s2 = p2.battleTag.clone() };
              let s3 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
                  , p1.unitScore.unitsProduced
                  , p1.unitScore.unitsKilled
                  , p1.resourceScore.goldCollected
                  , p1.heroScore.expGained);
              let s4 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
                  , p2.unitScore.unitsProduced
                  , p2.unitScore.unitsKilled
                  , p2.resourceScore.goldCollected
                  , p2.heroScore.expGained);
              let scores = if m.teams[0].players[0].battleTag == s1 {
                  Some((s1,s2,s3,s4))
                } else {
                  Some((s2,s1,s4,s3))
                };
              return Some(FinishedGame
                { desc: mstr
                , passed_time: duration_in_minutes
                , win: are_you_winning
                , additional_fields: scores
                });
            }
            return Some(FinishedGame
              { desc: mstr
              , passed_time: duration_in_minutes
              , win: are_you_winning
              , additional_fields: None
              });
          }, None => {
            return None;
          }
        }
      }, Err(err) => {
        error!("Failed parse MD {:?}", err);
      }
    }
  }
  None
}

pub async fn check<'a>( ctx: &Context
                      , channel_id: u64
                      , guild_id: u64
                      , games_lock: &mut MutexGuard<'a, HashMap<String, TrackingGame>>
                      ) -> Vec<StartingGame> {
  let mut out : Vec<StartingGame> = Vec::new();
  if let Ok(res) =
    // getaway 20 = Europe
    reqwest::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20").await {
    if let Ok(going) = res.json::<Going>().await {
      if !going.matches.is_empty() {
        for m in going.matches {
          if m.gameMode == 1 {
            if m.teams.len() > 1 && !m.teams[0].players.is_empty() && !m.teams[1].players.is_empty() {
              if let Some(playa) = teammates().into_iter().find(|p|
                   m.teams[0].players[0].battleTag == p.battletag
                || m.teams[1].players[0].battleTag == p.battletag
              ) {
                set!{ g_map = get_map(m.map.as_str())
                    , race1 = get_race2(m.teams[0].players[0].race)
                    , race2 = get_race2(m.teams[1].players[0].race) };
                let mstr = format!("({}) **{}** [{}] *vs* ({}) **{}** [{}] *{}*",
                  race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);

                if let Some(track) = games_lock.get_mut(m.startTime.as_str()) {
                  track.still_live = true;
                  let minutes = track.passed_time / 2;
                  let footer = format!("Passed: {} min", minutes);

                  if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id).await {
                    if let Ok(user) = ctx.http.get_user(playa.discord).await {

                      let mut fields = Vec::new();
                      let mut img = None;
                      let mut url = None;
                      let mut color = (32,32,32);
                      if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                        for f in msg.embeds[0].fields.clone() {
                          fields.push((f.name, f.value, f.inline));
                        }
                        img = msg.embeds[0].image.clone();
                        url = msg.embeds[0].url.clone();
                        color = msg.embeds[0].colour.tuple();
                      };

                      if let Err(why) = msg.edit(ctx, |m| m
                        .embed(|e|  {
                          let mut e = e
                            .title("LIVE")
                            .author(|a| a.icon_url(&user.face()).name(&user.name))
                            .description(mstr)
                            .colour(color)
                            .footer(|f| f.text(footer));
                          if !fields.is_empty() {
                            e = e.fields(fields);
                          }
                          if let Some(some_img) = img {
                            e = e.image(some_img.url);
                          }
                          if let Some(some_url) = url {
                            e = e.url(some_url);
                          }
                          e
                        }
                      )).await {
                        error!("Failed to post live match {:?}", why);
                      }
                    }
                  }

                } else {
                  out.push(
                    StartingGame {
                      key: m.startTime,
                      description: mstr,
                      player: playa
                    }
                  );
                }
              }
            }
          } else if m.gameMode == 6 || m.gameMode == 2 { // AT or RT mode
            if m.teams.len() > 1 && m.teams[0].players.len() > 1 && m.teams[1].players.len() > 1 {
              if let Some(playa) = teammates().into_iter().find(|p|
                   m.teams[0].players[0].battleTag == p.battletag
                || m.teams[1].players[0].battleTag == p.battletag
                || m.teams[0].players[1].battleTag == p.battletag
                || m.teams[1].players[1].battleTag == p.battletag) {

                let g_map = get_map(m.map.as_str());

                set! { race1  = get_race2(m.teams[0].players[0].race)
                     , race12 = get_race2(m.teams[0].players[1].race)
                     , race2  = get_race2(m.teams[1].players[0].race)
                     , race22 = get_race2(m.teams[1].players[1].race) };

                let mstr = if m.gameMode == 6 {
                  format!("({}+{}) **{}** + **{}** [{}]\n*vs*\n({}+{}) **{}** + **{}** [{}]\n\nmap: **{}**",
                    race1, race12, m.teams[0].players[0].name, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr
                  , race2, race22, m.teams[1].players[0].name, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr, g_map)
                } else {
                  format!("({}+{}) **{}** [{}] + **{}** [{}]\n*vs*\n({}+{}) **{}** [{}] + **{}** [{}]\n\nmap: **{}**",
                    race1, race12, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr
                  , race2, race22, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr, g_map)
                };

                if let Some(track) = games_lock.get_mut(m.startTime.as_str()) {
                  track.still_live = true;
                  set!{ minutes = track.passed_time / 2
                      , footer = format!("Passed: {} min", minutes) };
                  if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id).await {
                    if let Ok(user) = ctx.http.get_user(playa.discord).await {
                      setm!{ fields = Vec::new()
                           , img    = None
                           , url    = None
                           , color = (32,32,32) };
                      if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                        for f in msg.embeds[0].fields.clone() {
                          fields.push((f.name, f.value, f.inline));
                        }
                        img = msg.embeds[0].image.clone();
                        url = msg.embeds[0].url.clone();
                        color = msg.embeds[0].colour.tuple();
                      };

                      if let Err(why) = msg.edit(ctx, |m| m
                        .embed(|e| {
                          let mut e = e
                            .title("LIVE")
                            .author(|a| a.icon_url(&user.face()).name(&user.name))
                            .description(mstr)
                            .colour(color)
                            .footer(|f| f.text(footer));
                          if !fields.is_empty() {
                            e = e.fields(fields);
                          }
                          if let Some(some_img) = img {
                            e = e.image(some_img.url);
                          }
                          if let Some(some_url) = url {
                            e = e.url(some_url);
                          }
                          e
                        }
                      )).await {
                        error!("Failed to post live match {:?}", why);
                      }
                    }
                  }
                } else {
                  out.push(
                    StartingGame {
                      key: m.startTime,
                      description: mstr,
                      player: playa
                    }
                  );
                }

              }
            }
          }
        }

        let mut k_to_del : Vec<String> = Vec::new();
        for (k, track) in games_lock.iter_mut() {
          if !track.still_live {
            if let Some(finished_game) =
                check_match(k, track.player.battletag.as_str()).await {
              let fgame = &finished_game;
              if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id).await {
                let footer : String = format!("Passed: {} min", fgame.passed_time);
                if let Ok(user) = ctx.http.get_user(track.player.discord).await {
                  let mut old_fields = Vec::new();
                  let mut url = None;
                  let mut color = (32,32,32);
                  if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                    for f in msg.embeds[0].fields.clone() {
                      old_fields.push((f.name, f.value, f.inline));
                    }
                    url = msg.embeds[0].url.clone();
                    color = msg.embeds[0].colour.tuple();
                  };
                  let mut title = "FINISHED";
                  let mut streak_fields = None;
                  if fgame.win {
                    info!("Registering win for {}", user.name);
                    let streak = points::add_win_points( guild_id
                                                       , track.player.discord
                                                       ).await;
                    if streak >= 3 {
                      title =
                        match streak {
                          3  => "MULTIKILL",
                          4  => "MEGA KILL",
                          5  => "ULTRAKILL",
                          6  => "KILLING SPREE",
                          7  => "RAMPAGE!",
                          8  => "DOMINATING",
                          9  => "UNSTOPPABLE",
                          10 => "GODLIKE!",
                          11 => "WICKED SICK",
                          12 => "ALPHA",
                          _  => "FRENETIC"
                        };
                      let dd = format!("Doing _**{}**_ kills in a row**!**", streak);
                      streak_fields = Some(vec![("Winning streak", dd, false)]);
                    }
                  } else {
                    info!("Registering lose for {}", user.name);
                    points::break_streak( guild_id
                                        , track.player.discord ).await;
                  }
                  if let Err(why) = msg.edit(ctx, |m| m
                    .embed(|e| {
                      let mut e =
                        e.author(|a| a.icon_url(&user.face()).name(&user.name))
                        .title(title)
                        .description(fgame.desc.as_str())
                        .colour(color)
                        .footer(|f| f.text(footer));
                      if !old_fields.is_empty() {
                        e = e.fields(old_fields);
                      }
                      if let Some(streak_data) = streak_fields {
                        e = e.fields(streak_data);
                      }
                      if let Some((s1,s2,s3,s4)) = &fgame.additional_fields {
                        e = e.fields(vec![
                          (s1, s3, true),
                          (s2, s4, true)
                        ]);
                      }
                      if let Some(some_url) = url {
                        e = e.url(some_url);
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
            }
          }
        }

        for ktd in k_to_del {
          games_lock.remove(ktd.as_str());
        }

      }
    }
  }
  out
}

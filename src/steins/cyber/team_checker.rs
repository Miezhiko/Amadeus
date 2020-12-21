use crate::{
  types::{ common::{ CoreGuild, CoreGuilds }
         , team::Player
         , tracking::*
         , w3c::{ Going, MD } },
  collections::team::teammates,
  common::db::trees,
  steins::cyber::{
    utils::{ get_race2
           , get_map
           , get_hero_png }
  },
  steins::ai::chain
};

use serenity::{
  prelude::*,
  model::id::{ UserId
             , GuildId }
};

use std::collections::HashMap;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

pub static GAMES: Lazy<Mutex<HashMap<String, TrackingGame>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

async fn check_match( matchid: &str
                    , playaz: &[Player]
                    , rqcl: &reqwest::Client
                    ) -> Option<FinishedGame> {
  let url =
    format!("https://statistic-service.w3champions.com/api/matches/by-ongoing-match-id/{}", matchid);

  let mut if_md : Option<MD> = None;

  if let Ok(res) = rqcl.get(&url).send().await {
    match res.json::<MD>().await {
      Ok(md) => {
        if_md = Some(md);
      }, Err(err) => {
        warn!("Failed parse by-ongoing-match {:?}", err);
      }
    }
  }

  // fallback mode when by-ongoing-match-id fails
  if if_md.is_none() {
    if let Ok(wtf) = rqcl.get("https://statistic-service.w3champions.com/api/matches?offset=0")
                         .send()
                         .await {
      if let Ok(going) = wtf.json::<Going>().await {
        for mm in &going.matches {
          if mm.match_id == matchid {
            let url = format!("https://statistic-service.w3champions.com/api/matches/{}", mm.id);
            if let Ok(res) = rqcl.get(&url).send().await {
              match res.json::<MD>().await {
                Ok(md) => {
                  if_md = Some(md);
                  break;
                }, Err(err) => {
                  error!("Failed parse match/id result {:?}", err);
                }
              }
            }
          }
        }
      }
    }
  }

  if let Some(md) = if_md {
    let m = md.match_data;
    let address = format!("https://www.w3champions.com/match/{}", &m.id);
    let mut losers: Vec<(u64, bool)> = vec![];
    let mstr_o =
      if m.gameMode == 1 {
        set!{ g_map = get_map(&m.map)
            , race1 = get_race2(m.teams[0].players[0].race)
            , race2 = get_race2(m.teams[1].players[0].race) };
        for i in 0..2 {
          if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[0].battleTag == p.battletag) {
            let won = m.teams[i].players[0].won;
            losers.push((playa.discord, won));
          }
        }
        let player1 = if m.teams[0].players[0].won {
          format!("__**{}**__ **+{}**", m.teams[0].players[0].name, m.teams[0].players[0].mmrGain)
        } else {
          format!("__*{}*__ **{}**", m.teams[0].players[0].name, m.teams[1].players[0].mmrGain)
        };
        let player2 = if m.teams[1].players[0].won {
          format!("__**{}**__ **+{}**", m.teams[1].players[0].name, m.teams[0].players[0].mmrGain)
        } else {
          format!("__*{}*__ **{}**", m.teams[1].players[0].name, m.teams[1].players[0].mmrGain)
        };
        Some(
          vec![ format!("({}) {} [{}] *vs* ({}) {} [{}] *{}*",
                race1, player1, m.teams[0].players[0].oldMmr
              , race2, player2, m.teams[1].players[0].oldMmr, g_map) ])
      } else if m.gameMode == 6 || m.gameMode == 2 {
        let g_map  = get_map(&m.map);
        for i in 0..2 {
          for j in 0..2 {
            if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[j].battleTag == p.battletag) {
              let won = m.teams[i].players[j].won;
              losers.push((playa.discord, won));
            }
          }
        }
        let mstr = format!("Map: {}", g_map);
        let teamx = |x: usize| -> String {
          if m.gameMode == 6 {
            if m.teams[x].won {
              format!("({}) __**{}**__\n({}) __**{}**__\n[{}] **+{}**"
              , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name
              , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
            } else {
              format!("({}) __*{}*__\n({}) __*{}*__\n[{}] *{}*"
              , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name
              , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
            }
          } else if m.teams[x].won {
            format!("({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**"
            , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name, m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
          } else {
            format!("({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*"
            , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name, m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
          }
        };
        Some( vec![ mstr, teamx(0), teamx(1) ] )
      } else if m.gameMode == 4 {
        let g_map  = get_map(&m.map);
        for i in 0..2 {
          for j in 0..4 {
            if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[j].battleTag == p.battletag) {
              let won = m.teams[i].players[j].won;
              losers.push((playa.discord, won));
            }
          }
        }
        let mstr = format!("Map: {}", g_map);
        let teamx = |x: usize| -> String {
          if m.teams[x].won {
            format!("({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**"
            , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name, m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain
            , get_race2(m.teams[x].players[2].race), m.teams[x].players[2].name, m.teams[x].players[2].oldMmr, m.teams[x].players[2].mmrGain
            , get_race2(m.teams[x].players[3].race), m.teams[x].players[3].name, m.teams[x].players[3].oldMmr, m.teams[x].players[3].mmrGain)
          } else {
            format!("({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*"
            , get_race2(m.teams[x].players[0].race), m.teams[x].players[0].name, m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), m.teams[x].players[1].name, m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain
            , get_race2(m.teams[x].players[2].race), m.teams[x].players[2].name, m.teams[x].players[2].oldMmr, m.teams[x].players[2].mmrGain
            , get_race2(m.teams[x].players[3].race), m.teams[x].players[3].name, m.teams[x].players[3].oldMmr, m.teams[x].players[3].mmrGain)
          }
        };
        Some( vec![ mstr, teamx(0), teamx(1) ] )
      } else {
        None
      };
    if let Some(mstr) = mstr_o {
      let mut maybe_hero_png = None;
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

        // To display hero icon / scores we use 1st playa
        let btag = &playaz[0].battletag;
        let player_scores =
          if btag == &s1 {
            &md.playerScores[0]
          } else {
            &md.playerScores[1]
          };
        let scores = if m.teams[0].players[0].battleTag == s1 {
            Some((s1,s2,s3,s4))
          } else {
            Some((s2,s1,s4,s3))
          };
        if !player_scores.heroes.is_empty() {
          maybe_hero_png = Some(get_hero_png(
            &player_scores.heroes[0].icon)
          );
        }
        return Some(FinishedGame
          { desc: mstr
          , passed_time: duration_in_minutes
          , link: address
          , winners: losers
          , additional_fields: scores
          , hero_png: maybe_hero_png
          });
      } else if (m.gameMode == 6 || m.gameMode == 2) && md.playerScores.len() > 3 {
        // Again, to display hero icon / scores we use 1st playa
        let btag = &playaz[0].battletag;
        let player_scores =
          if let Some(scores) = &md.playerScores.iter().find(|s| {
            &s.battleTag == btag
          }) { scores } else { &md.playerScores[0] };
        if !player_scores.heroes.is_empty() {
          maybe_hero_png = Some(get_hero_png(
            &player_scores.heroes[0].icon)
          );
        }
        // for 2x2 mode display scores of teammate
        // or if two or more clan players in then clan players
        let teammate_scores =
          if playaz.len() > 1 {
            if let Some(scores) = &md.playerScores.iter().find(|s| {
              s.battleTag == playaz[1].battletag
            }) { scores } else { &md.playerScores[1] }
          } else if let Some(team) = m.teams.iter().find(|t| {
            t.players.iter().any(|p| {
                &p.battleTag == btag
              })
            }) {
            if let Some(not_me) = team.players.iter().find(|p| {
              &p.battleTag != btag
            }) {
              if let Some(scores) = &md.playerScores.iter().find(|s| {
                s.battleTag == not_me.battleTag
              }) {
                scores
              } else { &md.playerScores[1] }
            } else { &md.playerScores[1] }
          } else { &md.playerScores[1] };
        set! { s1 = player_scores.battleTag.clone()
             , s2 = teammate_scores.battleTag.clone() };
        let s3 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
            , player_scores.unitScore.unitsProduced
            , player_scores.unitScore.unitsKilled
            , player_scores.resourceScore.goldCollected
            , player_scores.heroScore.expGained);
        let s4 = format!("produced: {}\nkilled: {}\ngold: {}\nhero exp: {}"
            , teammate_scores.unitScore.unitsProduced
            , teammate_scores.unitScore.unitsKilled
            , teammate_scores.resourceScore.goldCollected
            , teammate_scores.heroScore.expGained);
        return Some(FinishedGame
          { desc: mstr
          , passed_time: duration_in_minutes
          , link: address
          , winners: losers
          , additional_fields: Some((s1,s2,s3,s4))
          , hero_png: maybe_hero_png
          });
      } else if m.gameMode == 4 {
        let btag = &playaz[0].battletag;
        let player_scores =
          if let Some(scores) = &md.playerScores.iter().find(|s| {
            &s.battleTag == btag
          }) { scores } else { &md.playerScores[0] };
        if !player_scores.heroes.is_empty() {
          maybe_hero_png = Some(get_hero_png(
            &player_scores.heroes[0].icon)
          );
        }
      }
      return Some(FinishedGame
        { desc: mstr
        , passed_time: duration_in_minutes
        , link: address
        , winners: losers
        , additional_fields: None
        , hero_png: maybe_hero_png
      });
    }
  }
  None
}

async fn generate_bet_fields( ctx: &Context
                            , track: &TrackingGame
                            ) -> Option<Vec<(String, String, bool)>> {
  let mut bet_fields = None;
  if !track.bets.is_empty() {
    let mut woutput = vec![];
    let mut loutput = vec![];
    for bet in &track.bets {
      let user_id = UserId( bet.member );
      if let Ok(user) = user_id.to_user(ctx).await {
        if bet.positive {
          woutput.push(
            format!("**{}**: {}", user.name, bet.points)
          );
        } else {
          loutput.push(
            format!("**{}**: {}", user.name, bet.points)
          );
        }
      }
    }
    let mut fstring = woutput.join("\n");
    if !loutput.is_empty() {
      let need_space = if woutput.is_empty() { "" } else { "\n" };
      fstring = format!("{}{}*on lose:*\n{}", fstring
                                            , need_space
                                            , loutput.join("\n"));
    }
    bet_fields = Some(vec![("Bets".to_string()
                          , fstring
                          , false)]);
  }
  bet_fields
}

pub async fn check<'a>( ctx: &Context
                      , channel_id: u64
                      , guild_id: u64
                      , rqcl: &reqwest::Client
                      ) -> Vec<StartingGame> {
  let mut out : Vec<StartingGame> = Vec::new();
  if let Ok(res) =
    rqcl.get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0")
        .send()
        .await {
    if let Ok(going) = res.json::<Going>().await {
      let guild = GuildId( guild_id );
      if !going.matches.is_empty() {
        for m in going.matches {
          if m.gameMode == 1 {
            if m.teams.len() > 1 && !m.teams[0].players.is_empty() && !m.teams[1].players.is_empty() {
              let playaz = teammates().into_iter().filter( |p|
                   m.teams[0].players[0].battleTag == p.battletag
                || m.teams[1].players[0].battleTag == p.battletag ).collect::<Vec<Player>>();
              if !playaz.is_empty() {
                set!{ g_map = get_map(&m.map)
                    , race1 = get_race2(m.teams[0].players[0].race)
                    , race2 = get_race2(m.teams[1].players[0].race) };
                let mstr = format!("({}) **{}** [{}] *vs* ({}) **{}** [{}] *{}*",
                    race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                  , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);

                { // games lock scope
                  let mut games_lock = GAMES.lock().await;
                  if let Some(track) = games_lock.get_mut(&m.match_id) {
                    track.still_live = true;
                    let minutes = track.passed_time / 2;
                    let footer = format!("Passed: {} min", minutes);

                    // use first player for discord operations
                    let playa = playaz[0].discord;
                    if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id[0]).await {
                      if let Ok(user) = ctx.http.get_user(playa).await {

                        let mut fields = Vec::new();
                        let mut img = None;
                        let mut url = None;
                        let mut color = (32,32,32);
                        if !msg.embeds.is_empty() {
                          if !msg.embeds[0].fields.is_empty() {
                            for f in msg.embeds[0].fields.clone() {
                              if f.name != "Bets" {
                                fields.push((f.name, f.value, f.inline));
                              }
                            }
                          }
                          img = msg.embeds[0].image.clone();
                          url = msg.embeds[0].url.clone();
                          color = msg.embeds[0].colour.tuple();
                        };

                        let bet_fields = generate_bet_fields(ctx, track).await;
                        let nick = user.nick_in(&ctx.http, guild)
                                       .await.unwrap_or_else(|| user.name.clone());

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e|  {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&nick))
                              .description(mstr)
                              .colour(color)
                              .footer(|f| f.text(footer));
                            if !fields.is_empty() {
                              e = e.fields(fields);
                            }
                            if let Some(bet_data) = bet_fields {
                              e = e.fields(bet_data);
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
                      StartingGame { key: m.match_id
                                   , description: vec![ mstr ]
                                   , players: playaz });
                  }
                }
              }
            }
          } else if m.gameMode == 6 || m.gameMode == 2 { // AT or RT mode
            if m.teams.len() > 1 && m.teams[0].players.len() > 1 && m.teams[1].players.len() > 1 {
              let playaz = teammates().into_iter().filter( |p|
                   m.teams[0].players[0].battleTag == p.battletag
                || m.teams[1].players[0].battleTag == p.battletag
                || m.teams[0].players[1].battleTag == p.battletag
                || m.teams[1].players[1].battleTag == p.battletag ).collect::<Vec<Player>>();

              if !playaz.is_empty() {
                let g_map = get_map(&m.map);

                set! { race1  = get_race2(m.teams[0].players[0].race)
                     , race12 = get_race2(m.teams[0].players[1].race)
                     , race2  = get_race2(m.teams[1].players[0].race)
                     , race22 = get_race2(m.teams[1].players[1].race) };

                let mstr = format!("Map: {}", g_map);

                let mvec =
                  if m.gameMode == 6 {
                    let team1 = format!("({}) **{}**\n({}) **{}**\n{} MMR"
                      , race1, m.teams[0].players[0].name
                      , race12, m.teams[0].players[1].name, m.teams[0].players[0].oldMmr);
                    let team2 = format!("({}) **{}**\n({}) **{}**\n{} MMR"
                      , race2, m.teams[1].players[0].name
                      , race22, m.teams[1].players[1].name, m.teams[1].players[0].oldMmr);
                    vec![ mstr, team1, team2 ]
                  } else {
                    let team1 = format!("({}) **{}** [{}]\n({}) **{}** [{}]"
                      , race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                      , race12, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr);
                    let team2 = format!("({}) **{}** [{}]\n({}) **{}** [{}]"
                      , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr
                      , race22, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr);
                    vec![ mstr, team1, team2 ]
                  };

                { // games lock scope
                  let mut games_lock = GAMES.lock().await;
                  if let Some(track) = games_lock.get_mut(&m.match_id) {
                    track.still_live = true;
                    set!{ minutes = track.passed_time / 2
                        , footer = format!("Passed: {} min", minutes) };
                    if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id[0]).await {
                      // get first player for discord
                      let playa = playaz[0].discord;
                      if let Ok(user) = ctx.http.get_user(playa).await {
                        setm!{ fields = Vec::new()
                            , img    = None
                            , url    = None
                            , color  = (32,32,32) };
                        if !msg.embeds.is_empty() {
                          if !msg.embeds[0].fields.is_empty() {
                            for f in msg.embeds[0].fields.clone() {
                              if f.name != "Bets" {
                                fields.push((f.name, f.value, f.inline));
                              }
                            }
                          }
                          img = msg.embeds[0].image.clone();
                          url = msg.embeds[0].url.clone();
                          color = msg.embeds[0].colour.tuple();
                        };

                        let bet_fields = generate_bet_fields(ctx, track).await;
                        let nick = user.nick_in(&ctx.http, guild)
                                       .await.unwrap_or_else(|| user.name.clone());

                        if let Err(why) = msg.edit(ctx, |m| m
                          .embed(|e| {
                            let mut e = e
                              .title("LIVE")
                              .author(|a| a.icon_url(&user.face()).name(&nick))
                              .description(&mvec[0])
                              .colour(color)
                              .footer(|f| f.text(footer));
                            if !fields.is_empty() {
                              e = e.fields(fields);
                            }
                            if let Some(bet_data) = bet_fields {
                              e = e.fields(bet_data);
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
                      StartingGame { key: m.match_id
                                   , description: mvec
                                   , players: playaz });
                  }
                }
              }
            }
          } else if m.gameMode == 4 && // 4x4
            m.teams.len() > 1 && m.teams[0].players.len() > 3 && m.teams[1].players.len() > 3 {
            let playaz = teammates().into_iter().filter( |p|
                 m.teams[0].players[0].battleTag == p.battletag || m.teams[0].players[2].battleTag == p.battletag
              || m.teams[1].players[0].battleTag == p.battletag || m.teams[1].players[2].battleTag == p.battletag
              || m.teams[0].players[1].battleTag == p.battletag || m.teams[0].players[3].battleTag == p.battletag
              || m.teams[1].players[1].battleTag == p.battletag || m.teams[1].players[3].battleTag == p.battletag
              ).collect::<Vec<Player>>();

            if !playaz.is_empty() {
              let g_map = get_map(&m.map);

              set! { race1  = get_race2(m.teams[0].players[0].race), race13 = get_race2(m.teams[0].players[2].race)
                   , race12 = get_race2(m.teams[0].players[1].race), race14 = get_race2(m.teams[0].players[3].race)
                   , race2  = get_race2(m.teams[1].players[0].race), race23 = get_race2(m.teams[1].players[2].race)
                   , race22 = get_race2(m.teams[1].players[1].race), race24 = get_race2(m.teams[1].players[3].race) };

              let mstr = format!("Map: {}", g_map);

              let mvec = {
                  let team1 = format!("({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]"
                    , race1,  m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                    , race12, m.teams[0].players[1].name, m.teams[0].players[1].oldMmr
                    , race13, m.teams[0].players[2].name, m.teams[0].players[2].oldMmr
                    , race14, m.teams[0].players[3].name, m.teams[0].players[3].oldMmr);
                  let team2 = format!("({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]\n({}) **{}** [{}]"
                    , race2,  m.teams[1].players[0].name, m.teams[1].players[0].oldMmr
                    , race22, m.teams[1].players[1].name, m.teams[1].players[1].oldMmr
                    , race23, m.teams[1].players[2].name, m.teams[1].players[2].oldMmr
                    , race24, m.teams[1].players[3].name, m.teams[1].players[3].oldMmr);
                  vec![ mstr, team1, team2 ]
                };

              { // games lock scope
                let mut games_lock = GAMES.lock().await;
                if let Some(track) = games_lock.get_mut(&m.match_id) {
                  track.still_live = true;
                  set!{ minutes = track.passed_time / 2
                      , footer = format!("Passed: {} min", minutes) };
                  if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id[0]).await {
                    // get first player for discord
                    let playa = playaz[0].discord;
                    if let Ok(user) = ctx.http.get_user(playa).await {
                      setm!{ fields = Vec::new()
                           , img    = None
                           , url    = None
                           , color  = (32,32,32) };
                      if !msg.embeds.is_empty() {
                        if !msg.embeds[0].fields.is_empty() {
                          for f in msg.embeds[0].fields.clone() {
                            if f.name != "Bets" {
                              fields.push((f.name, f.value, f.inline));
                            }
                          }
                        }
                        img   = msg.embeds[0].image.clone();
                        url   = msg.embeds[0].url.clone();
                        color = msg.embeds[0].colour.tuple();
                      };

                      let bet_fields = generate_bet_fields(ctx, track).await;
                      let nick = user.nick_in(&ctx.http, guild)
                                     .await.unwrap_or_else(|| user.name.clone());

                      if let Err(why) = msg.edit(ctx, |m| m
                        .embed(|e| {
                          let mut e = e
                            .title("LIVE")
                            .author(|a| a.icon_url(&user.face()).name(&nick))
                            .description(&mvec[0])
                            .colour(color)
                            .footer(|f| f.text(footer));
                          if !fields.is_empty() {
                            e = e.fields(fields);
                          }
                          if let Some(bet_data) = bet_fields {
                            e = e.fields(bet_data);
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
                    StartingGame { key: m.match_id
                                 , description: mvec
                                 , players: playaz });
                }
              }
            }
          }
        }

        let mut k_to_del : Vec<String> = Vec::new();
        let mut games_lock = GAMES.lock().await;
        for (k, track) in games_lock.iter_mut() {
          if !track.still_live {
            if let Some(finished_game) =
                check_match(k, &track.players, rqcl).await {
              let fgame = &finished_game;
              if let Ok(mut msg) = ctx.http.get_message(channel_id, track.tracking_msg_id[0]).await {
                let footer : String = format!("Passed: {} min", fgame.passed_time);
                // git first player for discord (again, as ususal)
                let playa = track.players[0].discord;
                if let Ok(user) = ctx.http.get_user(playa).await {
                  let mut old_fields = Vec::new();
                  let mut color = (32,32,32);
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
                    color = msg.embeds[0].colour.tuple();
                  };
                  let mut title = "FINISHED";
                  let mut streak_fields = None;
                  let mut bet_fields = None;
                  for (pw, is_win) in &fgame.winners {
                    if *is_win {
                      trace!("Registering win for {}", pw);
                      let streak = trees::add_win_points( guild_id
                                                        , *pw
                                                        ).await;
                      if playa == *pw && streak >= 3 {
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
                        let dd = format!("Doing _**{}**_ kills in a row**!**", streak);
                        streak_fields = Some(vec![("Winning streak", dd, false)]);
                      }
                    } else {
                      trace!("Registering lose for {}", pw);
                      trees::break_streak(guild_id, *pw).await;
                    }
                    if !track.bets.is_empty() && bet_fields.is_none() {
                      trace!("Paying for bets");
                      let amadeus_maybe = {
                        let data = ctx.data.read().await;
                        if let Some(core_guilds) = data.get::<CoreGuilds>() {
                          if let Some(amadeus) = core_guilds.get(&CoreGuild::Amadeus) {
                            Some(*amadeus)
                          } else { None }
                        } else { None }
                      };
                      // There is complicated bet win calculation
                      if let Some(amadeus) = amadeus_maybe {
                        if let Ok(p) = trees::get_points( guild_id, amadeus ).await {
                          let mut win_calculation = HashMap::new();
                          let mut waste = 0;
                          let mut k : f32 = 2.0;
                          let mut losers_output = vec![];
                          for bet in &track.bets {
                            if *is_win == bet.positive {
                              let best_win = (bet.points as f32 * k).round() as u64;
                              win_calculation.insert(bet.member, (bet.points, best_win));
                              waste += best_win;
                            } else {
                              let user_id = UserId( bet.member );
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
                              trees::give_points( guild_id
                                                , amadeus
                                                , *mpp
                                                , *wpp ).await;
                            if !succ {
                              error!("failed to give bet win points: {}", rst);
                            } else {
                              let user_id = UserId( *mpp );
                              if let Ok(user) = user_id.to_user(&ctx).await {
                                let pure_win = *wpp - *ppp;
                                output.push(
                                  format!("**{}** wins **{}**", user.name, pure_win)
                                );
                              }
                            }
                          }
                          let title = format!("Bets coefficient: {}", k);
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
                  let tip =
                    if old_fields.is_empty() && streak_fields.is_none() && bet_fields.is_none()
                                             && fgame.additional_fields.is_some() {
                      Some(chain::generate_with_language(ctx, false).await)
                    } else { None };

                  let nick = user.nick_in(&ctx.http, guild)
                                 .await.unwrap_or_else(|| user.name.clone());

                  if let Err(why) = msg.edit(ctx, |m| m
                    .embed(|e| {
                      let mut e =
                        e.author(|a| a.icon_url(&user.face()).name(&nick))
                         .title(title)
                         .colour(color)
                         .url(&fgame.link)
                         .footer(|f| f.text(footer));
                      if !fgame.desc.is_empty() {
                        e = e.description(&fgame.desc[0]);
                        if fgame.desc.len() > 2 {
                          let d_fields = vec![
                            ("Team 1", &fgame.desc[1], true)
                          , ("Team 2", &fgame.desc[2], true)
                          ];
                          e = e.fields(d_fields);
                          // add line breaking something if there is no
                          if let Some(t) = tip {
                            e = e.fields(vec![
                              ("Tip for the day", &t, false)
                            ]);
                          }
                        }
                      }
                      if !old_fields.is_empty() {
                        e = e.fields(old_fields);
                      }
                      if let Some(streak_data) = streak_fields {
                        e = e.fields(streak_data);
                      }
                      if let Some(bet_data) = bet_fields {
                        e = e.fields(bet_data);
                      }
                      if let Some((s1,s2,s3,s4)) = &fgame.additional_fields {
                        e = e.fields(vec![
                          (s1, s3, true),
                          (s2, s4, true)
                        ]);
                      }
                      if let Some(hero) = &fgame.hero_png {
                        e.thumbnail(hero);
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
                if let Ok(msg) = ctx.http.get_message(channel_id, track.tracking_msg_id[0]).await {
                  if let Err(wtf) = msg.delete(ctx).await {
                    error!("Failed to clean up dropped Live game {:?}", wtf);
                  }
                }
              }
            }
          }
        }

        for ktd in k_to_del {
          games_lock.remove(&ktd);
        }

      }
    }
  }
  out
}

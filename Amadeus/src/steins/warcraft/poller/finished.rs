use crate::{
    types::{ team::DiscordPlayer
           , tracking::*
           , w3c::{ Going, MD } },
    common::constants::W3C_API,
    steins::warcraft::{
      aka_checker::{ aka, check_aka },
      utils::{ get_race2
             , get_map
             , get_hero_png }
    }
};

#[allow(clippy::needless_range_loop)]
pub async fn check_match( matchid: &str
                        , playaz: &[DiscordPlayer]
                        , track_mode: &GameMode
                        , rqcl: &reqwest::Client
                        ) -> Option<FinishedGame> {

  let url = format!("{W3C_API}/matches/by-ongoing-match-id/{matchid}");

  let mut if_md: Option<MD> = None;

  if let Ok(res) = rqcl.get(&url).send().await {
    match res.json::<MD>().await {
      Ok(md) => {
        if md.match_data.is_some() && md.playerScores.is_some() {
          if_md = Some(md);
        }
      }, Err(err) => {
        warn!("Failed parse by-ongoing-match {err}, url: {url}");
      }
    }
  }

  // fallback mode when by-ongoing-match-id fails
  if if_md.is_none() {
    let game_mode = match track_mode {
      GameMode::Solo  => "gameMode=1",
      GameMode::Team2 => "gameMode=2",
      GameMode::Team4 => "gameMode=4"
    };
    if let Ok(wtf) = rqcl.get(&format!("{W3C_API}/matches?offset=0&{game_mode}"))
                         .send()
                         .await {
      if let Ok(going) = wtf.json::<Going>().await {
        for mm in &going.matches {
          if mm.match_id == matchid {
            let url = format!("{W3C_API}/matches/{}", mm.id);
            if let Ok(res) = rqcl.get(&url).send().await {
              match res.json::<MD>().await {
                Ok(md) => {
                  if_md = Some(md);
                  break;
                }, Err(err) => {
                  error!("Failed parse match/id result {err}");
                }
              }
            }
          }
        }
      }
    }
  }

  if let Some(md) = if_md {
    let m = md.match_data?;
    let ps = md.playerScores?;
    let address = format!("https://www.w3champions.com/match/{}", &m.id);
    let mut losers: Winners = vec![];
    let mstr_o =
      if m.gameMode == 1 {
        set!{ g_map = get_map(&m.map)
            , race1 = get_race2(m.teams[0].players[0].race)
            , race2 = get_race2(m.teams[1].players[0].race) };
        for i in 0..2 {
          if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[0].battleTag == p.player.battletag
          || if !p.player.alt_accounts.is_empty() {
            p.player.alt_accounts.iter().any(|a| a == &m.teams[i].players[0].battleTag)
          } else { false }
          ) {
            let won = m.teams[i].players[0].won;
            let mmr = m.teams[i].players[0].currentMmr;
            losers.push(( ( playa.player.battletag.clone()
                          , playa.player.discord )
                        , won, mmr ));
          }
        }
        set!{ t0_name = aka(&m.teams[0].players[0], rqcl).await
            , t1_name = aka(&m.teams[1].players[0], rqcl).await };
        setm!{ t0_ping = String::new()
             , t1_ping = String::new() };
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

        let player1 = if m.teams[0].players[0].won {
          format!("__**{}**__ **+{}**", t0_name, m.teams[0].players[0].mmrGain)
        } else {
          format!("__*{}*__ **{}**", t0_name, m.teams[0].players[0].mmrGain)
        };
        let player2 = if m.teams[1].players[0].won {
          format!("__**{}**__ **+{}**", t1_name, m.teams[1].players[0].mmrGain)
        } else {
          format!("__*{}*__ **{}**", t1_name, m.teams[1].players[0].mmrGain)
        };
        Some( vec![ format!("Map: **{}**", g_map)
                  , format!("({}) {} [{}]{}", race1, player1, m.teams[0].players[0].oldMmr, t0_ping)
                  , format!("({}) {} [{}]{}", race2, player2, m.teams[1].players[0].oldMmr, t1_ping) ] )
      } else if m.gameMode == 6 || m.gameMode == 2 {
        let g_map  = get_map(&m.map);
        let mut aka_names: [[String; 2]; 2] = Default::default();
        for i in 0..2 {
          for j in 0..2 {
            if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[j].battleTag == p.player.battletag
              || if !p.player.alt_accounts.is_empty() {
                p.player.alt_accounts.iter().any(|a| a == &m.teams[i].players[j].battleTag)
              } else { false }
            ) {
              let won = m.teams[i].players[j].won;
              let mmr = m.teams[i].players[j].currentMmr;
              losers.push(( ( playa.player.battletag.clone()
                            , playa.player.discord )
                          , won, mmr ));
            }
            aka_names[i][j] = aka(&m.teams[i].players[j], rqcl).await;
          }
        }
        let teamx = |x: usize| -> String {
          if m.gameMode == 6 {
            if m.teams[x].won {
              format!("({}) __**{}**__\n({}) __**{}**__\n[{}] **+{}**"
              , get_race2(m.teams[x].players[0].race), aka_names[x][0]
              , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
            } else {
              format!("({}) __*{}*__\n({}) __*{}*__\n[{}] *{}*"
              , get_race2(m.teams[x].players[0].race), aka_names[x][0]
              , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
            }
          } else if m.teams[x].won {
            format!("({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**"
            , get_race2(m.teams[x].players[0].race), aka_names[x][0], m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
          } else {
            format!("({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*"
            , get_race2(m.teams[x].players[0].race), aka_names[x][0], m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain)
          }
        };
        Some( vec![ format!("Map: **{}**", g_map), teamx(0), teamx(1) ] )
      } else if m.gameMode == 4 {
        let g_map  = get_map(&m.map);
        let mut aka_names: [[String; 4]; 2] = Default::default();
        for i in 0..2 {
          for j in 0..4 {
            if let Some(playa) = playaz.iter().find(|p| m.teams[i].players[j].battleTag == p.player.battletag
              || if !p.player.alt_accounts.is_empty() {
                p.player.alt_accounts.iter().any(|a| a == &m.teams[i].players[0].battleTag)
              } else { false }
            ) {
              let won = m.teams[i].players[j].won;
              let mmr = m.teams[i].players[j].currentMmr;
              losers.push(( ( playa.player.battletag.clone()
                            , playa.player.discord )
                          , won, mmr ));
            }
            aka_names[i][j] = aka(&m.teams[i].players[j], rqcl).await;
          }
        }
        let teamx = |x: usize| -> String {
          if m.teams[x].won {
            format!("({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**\n({}) __**{}**__ [{}] **+{}**"
            , get_race2(m.teams[x].players[0].race), aka_names[x][0], m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain
            , get_race2(m.teams[x].players[2].race), aka_names[x][2], m.teams[x].players[2].oldMmr, m.teams[x].players[2].mmrGain
            , get_race2(m.teams[x].players[3].race), aka_names[x][3], m.teams[x].players[3].oldMmr, m.teams[x].players[3].mmrGain)
          } else {
            format!("({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*\n({}) __*{}*__ [{}] *{}*"
            , get_race2(m.teams[x].players[0].race), aka_names[x][0], m.teams[x].players[0].oldMmr, m.teams[x].players[0].mmrGain
            , get_race2(m.teams[x].players[1].race), aka_names[x][1], m.teams[x].players[1].oldMmr, m.teams[x].players[1].mmrGain
            , get_race2(m.teams[x].players[2].race), aka_names[x][2], m.teams[x].players[2].oldMmr, m.teams[x].players[2].mmrGain
            , get_race2(m.teams[x].players[3].race), aka_names[x][3], m.teams[x].players[3].oldMmr, m.teams[x].players[3].mmrGain)
          }
        };
        Some( vec![ format!("Map: **{}**", g_map), teamx(0), teamx(1) ] )
      } else {
        None
      };
    if let Some(mstr) = mstr_o {
      let mut maybe_hero_png = None;
      let duration_in_minutes = m.durationInSeconds / 60;
      if ps.len() > 1 && m.gameMode == 1 {
        set! { p1 = &ps[0]
             , p2 = &ps[1]
             , s1 = p1.battleTag.clone()
             , s2 = p2.battleTag.clone() };
        let s3 = format!("hero kills: {}\nexperience: {}\nproduced: {}\nkilled: {}\ngold: {}"
            , p1.heroScore.heroesKilled
            , p1.heroScore.expGained
            , p1.unitScore.unitsProduced
            , p1.unitScore.unitsKilled
            , p1.resourceScore.goldCollected);
        let s4 = format!("hero kills: {}\nexperience: {}\nproduced: {}\nkilled: {}\ngold: {}"
            , p2.heroScore.heroesKilled
            , p2.heroScore.expGained
            , p2.unitScore.unitsProduced
            , p2.unitScore.unitsKilled
            , p2.resourceScore.goldCollected);

        // To display hero icon / scores we use 1st playa
        let btag = &playaz[0].player.battletag;
        let player_scores =
          if btag == &s1
          || if !playaz[0].player.alt_accounts.is_empty() {
            playaz[0].player.alt_accounts.iter().any(|other_acc|
              other_acc == &s1
            )
          } else { false } {
            &ps[0]
          } else {
            &ps[1]
          };
        let a1 = if let Some(aka) = check_aka(&s1, rqcl).await
                  { aka } else if s1.contains('#') {
                      s1.split('#').collect::<Vec<&str>>()[0].to_string()
                    } else {
                      s1.clone()
                    };
        let a2 = if let Some(aka) = check_aka(&s2, rqcl).await
                  { aka } else if s2.contains('#') {
                      s2.split('#').collect::<Vec<&str>>()[0].to_string()
                    } else {
                      s2.clone()
                    };
        let scores = if m.teams[0].players[0].battleTag == s1 {
            Some((a1,a2,s3,s4))
          } else {
            Some((a2,a1,s4,s3))
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
      } else if (m.gameMode == 6 || m.gameMode == 2) && ps.len() > 3 {
        // Again, to display hero icon / scores we use 1st playa
        let btag = &playaz[0].player.battletag;
        let player_scores =
          if let Some(scores) = &ps.iter().find(|s| {
            &s.battleTag == btag
            || if !playaz[0].player.alt_accounts.is_empty() {
              playaz[0].player.alt_accounts.iter().any(|other_acc|
                other_acc == &s.battleTag
              )
            } else { false }
          }) { scores } else { &ps[0] };
        if !player_scores.heroes.is_empty() {
          maybe_hero_png = Some(get_hero_png(
            &player_scores.heroes[0].icon)
          );
        }
        // for 2x2 mode display scores of teammate
        // or if two or more clan players in then clan players
        let teammate_scores =
          if playaz.len() > 1 {
            if let Some(scores) = &ps.iter().find(|s| {
              s.battleTag == playaz[1].player.battletag
              || if !playaz[1].player.alt_accounts.is_empty() {
                playaz[1].player.alt_accounts.iter().any(|a| a == &s.battleTag)
              } else { false }
            }) { scores } else { &ps[1] }
          } else if let Some(team) = m.teams.iter().find(|t| {
            t.players.iter().any(|p| {
                &p.battleTag == btag
              })
            }) {
            if let Some(not_me) = team.players.iter().find(|p| {
              &p.battleTag != btag
            }) {
              if let Some(scores) = &ps.iter().find(|s| {
                s.battleTag == not_me.battleTag
              }) {
                scores
              } else { &ps[1] }
            } else { &ps[1] }
          } else { &ps[1] };

        setm!{ t0_ping = String::new()
             , t1_ping = String::new() };
        if m.serverInfo.playerServerInfos.len() > 3 {
          setm!{ t0_index = 0
               , t1_index = 1 };
          for i in 0..4 {
            if player_scores.battleTag == m.serverInfo.playerServerInfos[i].battleTag {
              t0_index = i;
            }
            else if teammate_scores.battleTag == m.serverInfo.playerServerInfos[i].battleTag {
              t1_index = i;
            }
          }
          t0_ping = format!("\navg ping: {}ms", m.serverInfo.playerServerInfos[t0_index].averagePing);
          t1_ping = format!("\navg ping: {}ms", m.serverInfo.playerServerInfos[t1_index].averagePing);
        }

        let s1 = if let Some(aka) = check_aka(&player_scores.battleTag, rqcl).await
                  { aka } else if player_scores.battleTag.contains('#') {
                      player_scores.battleTag.split('#').collect::<Vec<&str>>()[0].to_string()
                    } else {
                      player_scores.battleTag.clone()
                    };
        let s2 = if let Some(aka) = check_aka(&teammate_scores.battleTag, rqcl).await
                  { aka } else if teammate_scores.battleTag.contains('#') {
                      teammate_scores.battleTag.split('#').collect::<Vec<&str>>()[0].to_string()
                    } else {
                      teammate_scores.battleTag.clone()
                    };
        let s3 = format!("hero kills: {}{}\nexperience: {}\nproduced: {}\nkilled: {}\ngold: {}"
            , player_scores.heroScore.heroesKilled
            , t0_ping
            , player_scores.heroScore.expGained
            , player_scores.unitScore.unitsProduced
            , player_scores.unitScore.unitsKilled
            , player_scores.resourceScore.goldCollected);
        let s4 = format!("hero kills: {}{}\nexperience: {}\nproduced: {}\nkilled: {}\ngold: {}"
            , teammate_scores.heroScore.heroesKilled
            , t1_ping
            , teammate_scores.heroScore.expGained
            , teammate_scores.unitScore.unitsProduced
            , teammate_scores.unitScore.unitsKilled
            , teammate_scores.resourceScore.goldCollected);
        return Some(FinishedGame
          { desc: mstr
          , passed_time: duration_in_minutes
          , link: address
          , winners: losers
          , additional_fields: Some((s1,s2,s3,s4))
          , hero_png: maybe_hero_png
          });
      } else if m.gameMode == 4 {
        let btag = &playaz[0].player.battletag;
        let player_scores =
          if let Some(scores) = &ps.iter().find(|s| {
            &s.battleTag == btag
            || if !playaz[0].player.alt_accounts.is_empty() {
              playaz[0].player.alt_accounts.iter().any(|other_acc|
                other_acc == &s.battleTag
              )
            } else { false }
          }) { scores } else { &ps[0] };
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

use crate::{
  collections::team::{ DIVISION1, DIVISION2 },
  stains::pad::{
    types::*,
    utils::{ get_race2, get_map }
  }
};

use reqwest;

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
  pub static ref GAMES: Mutex<HashMap<String, (u64, u32)>> = Mutex::new(HashMap::new());
}

pub fn check() -> Option<(String, String)> {
  if let Ok(res) =
    reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20&pageSize=50&gameMode=1") {
    if let Ok(going) = res.json::<Going>() {
      if going.matches.len() > 0 {
        for m in going.matches {
          if m.teams.len() > 1 && m.teams[0].players.len() > 0 && m.teams[1].players.len() > 0 {
            let is_div1 = DIVISION1.into_iter().any(|u|
              m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
            );
            let is_div2 = DIVISION2.into_iter().any(|u|
              m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
            );
            if is_div1 || is_div2 {
              if let Ok(games_lock) = GAMES.lock() {
                if let Some(_stored) = games_lock.get(m.id.as_str()) {
                  ()
                } else {
                  let g_map = get_map(m.map.as_str());
                  let race1 = get_race2(m.teams[0].players[0].race);
                  let race2 = get_race2(m.teams[1].players[0].race);
                  let mstr = format!("({}) **{}** [{}] vs ({}) **{}** [{}] *{}*",
                    race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                  , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);
                  return Some((m.id, mstr));
                }
              }
            }
          }
        }
      }
    }
  }
  None
}

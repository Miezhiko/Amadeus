use crate::{
  collections::team::{ DIVISION1, DIVISION2 },
  stains::pad::{
    types::*,
    utils::{ get_race2, get_map }
  }
};

use serenity::{
  prelude::*
};

use reqwest;

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
  pub static ref GAMES: Mutex<HashMap<String, (u64, u32, bool)>> = Mutex::new(HashMap::new());
}

pub fn check_match(matchid : &str) -> Option<String> {
  let url = format!("https://statistic-service.w3champions.com/api/matches/{}", matchid);
  if let Ok(res) = reqwest::blocking::get(url.as_str()) {
    if let Ok(md) = res.json::<MD>() {
      info!("step 5");
      let m = md.r#match;
      let g_map = get_map(m.map.as_str());
      let race1 = get_race2(m.teams[0].players[0].race);
      let race2 = get_race2(m.teams[1].players[0].race);
      let player1 = if m.teams[0].players[0].won {
        format!("__**{}**__ **+{}**", m.teams[0].players[0].name, m.teams[0].players[0].mmrGain)
      } else {
        format!("__*{}*__ **{}**", m.teams[0].players[0].name, m.teams[0].players[0].mmrGain)
      };
      let player2 = if m.teams[1].players[0].won {
        format!("__**{}**__ **+{}**", m.teams[1].players[0].name, m.teams[0].players[0].mmrGain)
      } else {
        format!("__*{}*__ **{}**", m.teams[1].players[0].name, m.teams[0].players[0].mmrGain)
      };
      let mstr = format!("({}) {} [{}] vs ({}) {} [{}] *{}*",
        race1, player1, m.teams[0].players[0].oldMmr
      , race2, player2, m.teams[1].players[0].oldMmr, g_map);
      return Some(mstr);
    }
  }
  None
}

pub fn check(ctx : &Context, channel_id : u64) -> Vec<(String, String)> {
  let mut out : Vec<(String, String)> = Vec::new();
  if let Ok(res) =
    reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20&pageSize=50&gameMode=1") {
    if let Ok(going) = res.json::<Going>() {
      if going.matches.len() > 0 {
        if let Ok(mut games_lock) = GAMES.lock() {

          for m in going.matches {
            if m.teams.len() > 1 && m.teams[0].players.len() > 0 && m.teams[1].players.len() > 0 {
              let is_div1 = DIVISION1.into_iter().any(|u|
                m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
              );
              let is_div2 = DIVISION2.into_iter().any(|u|
                m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
              );
              if is_div1 || is_div2 {
                info!("step 4");
                let g_map = get_map(m.map.as_str());
                let race1 = get_race2(m.teams[0].players[0].race);
                let race2 = get_race2(m.teams[1].players[0].race);
                let mstr = format!("({}) **{}** [{}] vs ({}) **{}** [{}] *{}*",
                  race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);
                if let Some((_v1, _, g)) = games_lock.get_mut(m.id.as_str()) {
                  *g = true;
                  //TODO:
                  // possibly here it change id and I need to update HashMap
                  /*
                  if let Ok(mut msg) = ctx.http.get_message(channel_id, *v1) {
                    if let Err(why) = msg.edit(ctx, |m| m.content(mstr)) {
                      error!("Failed to update game score {:?}", why);
                    }
                  }
                  */
                } else {
                  out.push((m.id, mstr));
                }
              }
            }
          }

          let mut k_to_del : Vec<String> = Vec::new();
          for (k, (v1, _, v3)) in games_lock.iter_mut() {
            if !*v3 {
              if let Some(new_text) = check_match(k) {
                if let Ok(mut msg) = ctx.http.get_message(channel_id, *v1) {
                  if let Err(why) = msg.edit(ctx, |m| m.content(new_text)) {
                    error!("Failed to update game score {:?}", why);
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
  }
  out
}

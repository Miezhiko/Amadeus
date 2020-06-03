use crate::{
  collections::team::{ DIVISION1, DIVISION2, INTERESTING },
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
  pub static ref GAMES: Mutex<HashMap<String, (u64, u32, bool, u64)>> = Mutex::new(HashMap::new());
}

pub fn check_match(matchid : &str) -> Option<String> {
  let url = format!("https://statistic-service.w3champions.com/api/matches/{}", matchid);
  info!("Trying: {}", url);
  if let Ok(res) = reqwest::blocking::get(url.as_str()) {
    info!("step x6");
    match res.json::<MD>() {
      Ok(md) => {
        info!("step x7");
        let m = md.match_data;
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
      },
      Err(err) => {
        error!("Failed parse MD {:?}", err);
      }
    }
  }
  None
}

pub fn check(ctx : &Context, channel_id : u64) -> Vec<(String, String, u64)> {
  let mut out : Vec<(String, String, u64)> = Vec::new();
  if let Ok(res) =
    reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20&pageSize=50&gameMode=1") {
    if let Ok(going) = res.json::<Going>() {
      if going.matches.len() > 0 {
        if let Ok(mut games_lock) = GAMES.lock() {

          for m in going.matches {
            if m.teams.len() > 1 && m.teams[0].players.len() > 0 && m.teams[1].players.len() > 0 {
              let is_div1 = DIVISION1.into_iter().find(|(u, _)|
                m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
              );
              let is_div2 = DIVISION2.into_iter().find(|(u, _)|
                m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
              );
              let is_interesting = INTERESTING.into_iter().find(|(u, _)|
                m.teams[0].players[0].battleTag == *u || m.teams[1].players[0].battleTag == *u
              );
              let (s, u) = is_div1.unwrap_or(is_div2.unwrap_or(is_interesting.unwrap_or(&("", 0))));
              if !s.is_empty() && *u != 0 {
                info!("step 4");
                let g_map = get_map(m.map.as_str());
                let race1 = get_race2(m.teams[0].players[0].race);
                let race2 = get_race2(m.teams[1].players[0].race);
                let mstr = format!("({}) **{}** [{}] vs ({}) **{}** [{}] *{}*",
                  race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
                , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);

                if let Some((v1, v2, g, _)) = games_lock.get_mut(m.id.as_str()) {
                  *g = true;
                  let minutes = *v2 / 2;
                  let footer = format!("Passed: {} min", minutes);

                  if let Ok(mut msg) = ctx.http.get_message(channel_id, *v1) {
                    if let Ok(user) = ctx.http.get_user(*u) {
                      if let Err(why) = msg.edit(ctx, |m| m
                        .embed(|e| e
                        .author(|a| a.icon_url(&user.face()).name(&user.name))
                        .description(mstr)
                        .footer(|f| f.text(footer))
                      )) {
                          error!("Failed to post live match {:?}", why);
                      }
                    }
                  }

                } else {
                  out.push((m.id, mstr, *u));
                }

              }
            }
          }

          let mut k_to_del : Vec<String> = Vec::new();
          for (k, (v1, _, v3, u)) in games_lock.iter_mut() {
            if !*v3 {
              info!("step x1");
              if let Some(new_text) = check_match(k) {
                info!("step x2");
                if let Ok(mut msg) = ctx.http.get_message(channel_id, *v1) {
                  info!("step x3");

                  let mut footer : String = String::from("Passed: some min");
                  if msg.embeds.len() > 0 {
                    if let Some(foot) = &msg.embeds[0].footer {
                      footer = if !foot.text.is_empty() { foot.text.clone() } else { footer };
                    }
                  }

                  if let Ok(user) = ctx.http.get_user(*u) {
                    if let Err(why) = msg.edit(ctx, |m| m
                      .embed(|e| e
                      .author(|a| a.icon_url(&user.face()).name(&user.name))
                      .description(new_text)
                      .footer(|f| f.text(footer))
                    )) {
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
  }
  out
}

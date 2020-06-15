use crate::{
  common::{
    msg::{ channel_message }
  },
  stains::cyber::{
    types::*,
    utils::{ get_race, get_race2
           , get_league, get_map, get_league_png }
  }
};

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  },
};

use std::collections::HashMap;

use reqwest;
use comfy_table::*;

#[command]
pub fn ongoing(ctx: &mut Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  let res = reqwest::blocking::get("https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gateway=20&pageSize=50&gameMode=1")?;
  let going : Going = res.json()?;
  if going.matches.len() > 0 {
    let footer = format!("Requested by {}", msg.author.name);
    let mut description : String = String:: new();
    for m in going.matches.into_iter().take(15).collect::<Vec<Match>>() {
      if m.teams.len() > 1 && m.teams[0].players.len() > 0 && m.teams[1].players.len() > 0 {
        set! { g_map = get_map(m.map.as_str())
             , race1 = get_race2(m.teams[0].players[0].race)
             , race2 = get_race2(m.teams[1].players[0].race) };
        let mstr = format!("({}) **{}** [{}] vs ({}) **{}** [{}] *{}*",
          race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
        , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);
        description = format!("{}\n{}", mstr, description);
      }
    }
    if !description.is_empty() {
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title("Ongoing matches")
          .description(description)
          .thumbnail("https://i.pinimg.com/originals/b4/a0/40/b4a04082647a8505b3991cbaea7d2f86.png")
          .colour((180,40,200))
          .footer(|f| f.text(footer)))) {
        error!("Error sending ongoing message: {:?}", why);
      }
    }
  }
  Ok(())
}

#[command]
pub fn stats(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  let mut args_msg = args.message();
  if args_msg.is_empty() {
    args_msg = msg.author.name.as_str();
  }
  let userx = if args_msg.contains("#") { String::from(args_msg) }
    else {
      let search_uri = format!("https://statistic-service.w3champions.com/api/ladder/search?gateWay=20&searchFor={}&gameMode=1&season=1", args_msg);
      let ress = reqwest::blocking::get(search_uri.as_str())?;
      let search : Vec<Search> = ress.json()?;
      if search.len() > 0 {
        if search[0].player.playerIds.len() > 0 {
          search[0].player.playerIds[0].battleTag.clone()
        } else { String::from("") }
      } else { String::from("") }
    };
  if !userx.is_empty() {
    let user = userx.replace("#","%23");
    let game_mode_uri = format!("https://statistic-service.w3champions.com/api/players/{}/game-mode-stats?gateWay=20&season=1", user);
    let game_mode_res = reqwest::blocking::get(game_mode_uri.as_str())?;
    let game_mode_stats : Vec<GMStats> = game_mode_res.json()?;

    let mut league_info : String = String::new();
    let mut ffa_info : String = String::new();

    let mut at_list : Vec<(u32, String)> = Vec::new();
    let mut at_info : String = String::new();

    let mut league_avi : String = String::new();

    for gmstat in game_mode_stats {
      if gmstat.gameMode == 1 {
        let lid = gmstat.leagueOrder;
        let league_str = get_league(lid);
        league_avi = get_league_png(lid);
        let winrate = (gmstat.winrate * 100.0).round();
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("*League*: **{}** *Division:* **{}**", league_str, gmstat.division)
        } else {
          format!("*League*: **{}**", league_str)
        };
        let progr = if gmstat.rankingPointsProgress.mmr > 0 {
            format!("+{}", gmstat.rankingPointsProgress.mmr)
          } else {
            gmstat.rankingPointsProgress.mmr.to_string()
          };
        league_info = format!("**Winrate**: **{}%** **MMR**: __**{}**__ (*{}*)\n{} *Rank*: **{}**",
          winrate, gmstat.mmr, progr, league_division.as_str(), gmstat.rank);
      }
      if gmstat.gameMode == 5 {
        let lid = gmstat.leagueOrder;
        let league_str = get_league(lid);
        let winrate = (gmstat.winrate * 100.0).round();
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{}** *Division:* **{}**", league_str, gmstat.division)
        } else {
          format!("**{}**", league_str)
        };
        ffa_info = format!("{} *Rank*: **{}** *Winrate*: **{}%** *MMR*: __**{}**__",
          league_division, gmstat.rank, winrate, gmstat.mmr);
      }
      if gmstat.gameMode == 6 {
        let players = gmstat.playerIds;
        let mut player_str = String::new();
        for p in players {
          if p.battleTag != userx {
            player_str = p.name;
            break;
          }
        }
        let lid = gmstat.leagueOrder;
        let league_str = get_league(lid);
        let winrate = (gmstat.winrate * 100.0).round();
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{}** *div:* **{}**", league_str, gmstat.division)
        } else {
          format!("**{}**", league_str)
        };
        let strnfo = format!("__**{}**__ {} *gmaes* {} *Rank*: {} __**{}%**__ *MMR*: __**{}**__",
          player_str.as_str(), gmstat.games, league_division, gmstat.rank, winrate, gmstat.mmr);
        at_list.push((gmstat.mmr, strnfo));
      }
    }
    if at_list.len() > 0 {
      at_list.sort_by(|(mmra,_), (mmrb, _) | mmra.cmp(mmrb));
      at_list.reverse();
      let map_of_sort : Vec<String> = at_list.into_iter().map(|(_, strx)| strx).take(5).collect();
      if map_of_sort.len() > 0 {
        at_info = map_of_sort.join("\n");
      }
    }

    let uri = format!("https://statistic-service.w3champions.com/api/players/{}/race-stats?gateWay=20&season=1", user);
    let res = reqwest::blocking::get(uri.as_str())?;
    let stats : Vec<Stats> = res.json()?;

    let mut stats_by_races : String = String::new();
    if stats.len() > 0 {
      let name = &userx.split("#").collect::<Vec<&str>>()[0];
      for stat in &stats {
        let race = get_race(stat.race);
        let winrate = (stat.winrate * 100.0).round();
        stats_by_races = format!("{}\n**{}**\t : *wins*: {}, *loses*: {}, *winrate*: **{}%**", stats_by_races, race, stat.wins, stat.losses, winrate);
      }

      let max_games : Option<&Stats> = stats.iter().max_by_key(|s| s.games);
      let max_games_race = if max_games.is_some() { max_games.unwrap().race } else { 0 };
      let main_race_avatar = match max_games_race {
          1 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-3-icon.png",
          2 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-36/256/Warcraft-3-Reign-of-Chaos-5-icon.png",
          4 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-2-icon.png",
          8 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-icon.png",
          _ => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-31/256/Warcraft-II-new-2-icon.png"
        };
      let main_race_colors = match max_games_race {
          1 => (0, 0, 222),
          2 => (222, 0, 0),
          4 => (0, 222, 0),
          8 => (155, 0, 143),
          _ => (50, 120, 150)
        };

      let mut description = format!("[{}] {}\n", userx.as_str(), league_info.as_str());

      let uri2 = format!("https://statistic-service.w3champions.com/api/player-stats/{}/race-on-map-versus-race?season=1", user);
      let res2 = reqwest::blocking::get(uri2.as_str())?;
      let stats2 : Stats2 = res2.json()?;

      let mut table = Table::new();

      table.set_content_arrangement(ContentArrangement::Dynamic)
           .set_table_width(40)
           .set_header(vec!["Map", "vs HU", "vs O", "vs NE", "vs UD"]);

      let s24 = stats2.raceWinsOnMapByPatch.All;
      for s3 in s24 { //stats2.raceWinsOnMap {
        if s3.winLossesOnMap.len() > 0 {
          if s3.race == 16 { // max_games_race {
            for s4 in s3.winLossesOnMap {
              let text = get_map(s4.map.as_str());
              let mut scores : HashMap<u32, String> = HashMap::new();
              for s5 in s4.winLosses {
                let vs_winrate = (s5.winrate * 100.0).round();
                let text = format!("{}%", vs_winrate);
                scores.insert(s5.race, text);
              }
              table.add_row(vec![
                Cell::new(text).set_alignment(CellAlignment::Left),
                Cell::new(scores.get(&1).unwrap_or( &String::from("-") ))
                  .set_alignment(CellAlignment::Center),
                Cell::new(scores.get(&2).unwrap_or( &String::from("-") ))
                  .set_alignment(CellAlignment::Center),
                Cell::new(scores.get(&4).unwrap_or( &String::from("-") ))
                  .set_alignment(CellAlignment::Center),
                Cell::new(scores.get(&8).unwrap_or( &String::from("-") ))
                  .set_alignment(CellAlignment::Center)
              ]);
            }
          }
        }
      }

      description = format!("{}```\n{}\n```", description, table);

      let footer = format!("Requested by {}", msg.author.name);

      let mut additional_info = vec![("Stats by races", stats_by_races.as_str(), false)];
      if !ffa_info.is_empty() {
        additional_info.push(("FFA", ffa_info.as_str(), false));
      }
      if !at_info.is_empty() {
        additional_info.push(("AT 2x2", at_info.as_str(), false));
      }

      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title(name)
          .description(description)
          .thumbnail(if league_avi.is_empty() { main_race_avatar } else { league_avi.as_str() })
          .fields(additional_info)
          .colour(main_race_colors)
          .footer(|f| f.text(footer)))) {
        error!("Error sending stats message: {:?}", why);
      }
    } else {
      let resp = format!("User {} not found", args_msg);
      channel_message(&ctx, &msg, resp.as_str());
    }
  } else {
    let resp = format!("Search on {} found no users", args_msg);
    channel_message(&ctx, &msg, resp.as_str());
  }
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

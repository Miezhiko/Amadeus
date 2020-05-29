use crate::{
  common::{
    msg::{ channel_message }
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

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Stats {
  race: u32,
  gateWay: u32,
  id: String,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct WinLosses {
  race: u32,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct WinLossesOnMap {
  map: String,
  winLosses: Vec<WinLosses>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct RaceWinsOnMap {
  race: u32,
  winLossesOnMap: Vec<WinLossesOnMap>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Stats2 {
  id: String,
  raceWinsOnMap: Vec<RaceWinsOnMap>,
  battleTag: String,
  season: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct PlayerId {
  name: String,
  battleTag: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Player {
  playerIds: Vec<PlayerId>,
  name: String,
  id: String,
  mmr: u32,
  gateWay: u32,
  gameMode: u32,
  season: u32,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Search {
  gateway: u32,
  id: String,
  league: u32,
  rankNumber: u32,
  rankingPoints: u32,
  playerId: String,
  player: Player,
  gameMode: u32,
  season: u32,
  playersInfo: Option<String>
}

fn get_race(r : u32) -> String {
  String::from(
    match r {
      1 => "Human",
      2 => "Orc",
      4 => "Night Elf",
      8 => "Undead",
      _ => "Random"
    }
  )
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
    let uri = format!("https://statistic-service.w3champions.com/api/players/{}/race-stats?gateWay=20&season=1", user);
    let res = reqwest::blocking::get(uri.as_str())?;
    let stats : Vec<Stats> = res.json()?;
    
    let mut stats_by_races : String = String::new();
    if stats.len() > 0 {
      let name = &userx.split("#").collect::<Vec<&str>>()[0];
      for stat in &stats {
        let race = get_race(stat.race);
        let winrate = (stat.winrate * 100.0).round();
        stats_by_races = format!("{}\n**{}**\t : wins: {}, loses: {}, winrate: **{}%**", stats_by_races, race, stat.wins, stat.losses, winrate);
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

      let mut description = format!("[{}]\n", userx.as_str());

      let uri2 = format!("https://statistic-service.w3champions.com/api/player-stats/{}/race-on-map-versus-race?season=1", user);
      let res2 = reqwest::blocking::get(uri2.as_str())?;
      let stats2 : Stats2 = res2.json()?;

      let mut table = Table::new();

      table.set_content_arrangement(ContentArrangement::Dynamic)
           .set_table_width(40)
           .set_header(vec!["Map", "vs HU", "vs O", "vs NE", "vs UD"]);

      for s3 in stats2.raceWinsOnMap {
        if s3.winLossesOnMap.len() > 0 {
          if s3.race == 16 { // max_games_race {
            for s4 in s3.winLossesOnMap {
              let text = match s4.map.as_str() {
                "Overall"         => "All",
                "echoisles"       => "EI",
                "northernisles"   => "NIS",
                "amazonia"        => "AZ",
                "lastrefuge"      => "LR",
                "concealedhill"   => "CH",
                "twistedmeadows"  => "TM",
                "terenasstand"    => "TS",
                another_map       => another_map
              };
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
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title(name)
          .description(description)
          .thumbnail(main_race_avatar)
          .fields(vec![("Stats by races", stats_by_races.as_str(), false)])
          .colour(main_race_colors)
          .footer(|f| f.text(footer)))) {
        error!("Error sending help message: {:?}", why);
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

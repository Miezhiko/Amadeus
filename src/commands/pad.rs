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

use reqwest;

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

#[command]
pub fn stats(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  let args_msg = args.message();
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
    let mut out : Vec<(String, String, bool)> = Vec::new();
    if stats.len() > 0 {
      let name = &userx.split("#").collect::<Vec<&str>>()[0];
      for stat in &stats {
        let race = match stat.race {
          1 => "Human",
          2 => "Orc",
          4 => "Night Elf",
          8 => "Undead",
          _ => "Random"
        };
        let winrate = (stat.winrate * 100.0).round();
        let stat_str = format!("wins: {}, loses: {}, winrate: {}%", stat.wins, stat.losses, winrate);
        out.push((String::from(race), stat_str, false));
      }
      let max_games : Option<&Stats> = stats.iter().max_by_key(|p| p.games);
      let max_games_race = if max_games.is_some() { max_games.unwrap().race } else { 0 };
      let main_race_avatar = match max_games_race {
          1 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-3-icon.png",
          2 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-36/256/Warcraft-3-Reign-of-Chaos-5-icon.png",
          4 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-2-icon.png",
          8 => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-18/256/Warcraft-3-Reign-of-Chaos-icon.png",
          _ => "http://icons.iconarchive.com/icons/3xhumed/mega-games-pack-31/256/Warcraft-II-new-2-icon.png"
        };
        let footer = format!("Requested by {}", msg.author.name);
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title(name)
          .description(userx.as_str())
          .thumbnail(main_race_avatar)
          .fields(out)
          .colour((225, 222, 103))
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

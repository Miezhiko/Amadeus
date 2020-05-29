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

#[command]
pub fn stats(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  let userx = args.message();
  if !userx.is_empty() {
    let user = userx.replace("#","%23");
    let uri = format!("https://statistic-service.w3champions.com/api/players/{}/race-stats?gateWay=20&season=1", user);
    let res = reqwest::blocking::get(uri.as_str())?;
    let stats : Vec<Stats> = res.json()?;
    let mut out : Vec<(String, String, bool)> = Vec::new();
    if stats.len() > 0 {
      for stat in stats {
        let race = match stat.race {
          1 => "Human",
          2 => "Orc",
          4 => "Night Elf",
          8 => "Undead",
          _ => "Random"
        };
        let winrate = (stat.winrate * 100.0).round();
        let stat_str = format!("wins: {}, loses: {}, winrate: {}", stat.wins, stat.losses, winrate);
        out.push((String::from(race), stat_str, false));
      }
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title(userx)
          .thumbnail("https://upload.wikimedia.org/wikipedia/en/4/4f/Warcraft_III_Reforged_Logo.png")
          .fields(out)
          .colour((225, 222, 103)))) {
        error!("Error sending help message: {:?}", why);
      }
    }
  }
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

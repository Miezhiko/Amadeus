use crate::{
  types::tracking::{ W3CStats, GameMode },
  common::constants::{ W3C_STATS_ROOM, W3C_STATS_MSG },
  steins::warcraft::poller::GAMES,
  commands::w3c::{ generate_popularhours, get_mmm }
};

use chrono::{
  Timelike,
  Datelike
};
use serenity::prelude::*;
use std::collections::BTreeMap;
use async_std::fs;

const WEEKLY_STATS_FNAME: &str  = "weekly.yml";

#[derive(Clone, Serialize, Deserialize)]
pub struct WeeklyWinLoses {
  pub wins: u64,
  pub losses: u64
}

type WeeklyStats = BTreeMap<String, WeeklyWinLoses>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Weekly {
  pub reset_week: u32,
  pub statistics: WeeklyStats,
  pub statistics2: WeeklyStats,
  pub popular_hours: String
}

pub async fn get_weekly(ctx: &Context) -> anyhow::Result<Weekly> {
  if !std::path::Path::new(WEEKLY_STATS_FNAME).exists() {
    clear_weekly(ctx, chrono::Utc::now().iso_week().week()).await?;
  }
  let contents = fs::read_to_string(WEEKLY_STATS_FNAME).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn add_to_weekly(ctx: &Context, p: &str, win: bool, solo: bool) -> anyhow::Result<()> {
  let mut current_weekly = get_weekly(ctx).await?;
  let weekly_stats: &mut WeeklyStats =
    if solo { &mut current_weekly.statistics }
       else { &mut current_weekly.statistics2 };
  if let Some(p_stats) = weekly_stats.get_mut(p) {
    if win {
      p_stats.wins += 1;
    } else {
      p_stats.losses += 1;
    }
  } else {
    let stats = WeeklyWinLoses {
      wins    : if win { 1 } else { 0 },
      losses  : if win { 0 } else { 1 }
    };
    weekly_stats.insert(p.to_string(), stats);
  }
  let yml = serde_yaml::to_string(&current_weekly)?;
  fs::write(WEEKLY_STATS_FNAME, yml).await?;
  Ok(())
}

async fn clear_weekly(ctx: &Context, week: u32) -> anyhow::Result<()> {
  let poplar_hours =
    if let Some(generated_image) = generate_popularhours(ctx).await? {
      generated_image
    } else {
      "https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png".to_string()
    };
  let init = Weekly {
    reset_week: week,
    statistics: BTreeMap::new(),
    statistics2: BTreeMap::new(),
    popular_hours: poplar_hours
  };
  let yml = serde_yaml::to_string(&init)?;
  fs::write(WEEKLY_STATS_FNAME, yml).await?;
  Ok(())
}

pub async fn status_update(ctx: &Context, stats: &W3CStats) -> anyhow::Result<()> {
  if let Ok(mut statusmsg) = W3C_STATS_ROOM.message(ctx, W3C_STATS_MSG).await {
    let weekly = get_weekly(ctx).await?;
    let now = chrono::Utc::now();
    // only check on midnight (just because)
    if now.hour() == 0 {
      let now_week = now.iso_week().week();
      if now_week != weekly.reset_week {
        clear_weekly(ctx, now_week).await?;
      }
    }
    let (q1, q2, q3) = get_mmm(ctx).await?;
    let mut tracking_info = vec![];
    { // Games lock scope
      let games_lock = GAMES.lock().await;
      for game in games_lock.values() {
        if let Some(fp) = game.players.first() {
          let name = fp.player.battletag
                       .split('#')
                       .collect::<Vec<&str>>()[0];
          let game_mode_str = match game.mode {
            GameMode::Solo  => "1x1",
            GameMode::Team2 => "2x2",
            GameMode::Team4 => "4x4"
          };
          tracking_info.push(
            format!("{} play {} for {} mins"
            , name
            , game_mode_str
            , game.passed_time)
          );
        }
      }
    }
    let tracking_str = 
      if tracking_info.is_empty() {
        String::from("currently no games")
      } else {
        tracking_info.join("\n")
      };
    let mut weekly_str = vec![];
    for ws in &[weekly.statistics, weekly.statistics2] {
      weekly_str.push(
        if ws.is_empty() {
          String::from("no weekly statistic")
        } else {
          let mut weekly_vec = vec![];
          for (p, d) in ws {
            let name = p.split('#')
                        .collect::<Vec<&str>>()[0];
            let winrate = ( (d.wins as f32 / (d.wins + d.losses) as f32) * 100.0).round();
            weekly_vec.push(
              format!( "{}: {}W, {}L, {}%"
                    , name
                    , d.wins
                    , d.losses
                    , winrate )
            );
          }
          weekly_vec.join("\n")
        }
      );
    }
    let stats_str = format!(
"
__**weekly solo:**__
```
{}
```
__**weekly team games:**__
```
{}
```
__**currently running:**__
```
SOLO searching for: {}, GAMES: {}
2x2 searching for: {}, GAMES: {}
4x4 searching for: {}, GAMES: {}
```
__**currently playing:**__
```
{}
```
__**2x2 popular hours:**__"
    , weekly_str[0]
    , weekly_str[1]
    , q1, stats.games_solo
    , q2, stats.games_2x2
    , q3, stats.games_4x4
    , tracking_str);
    statusmsg.edit(ctx, |m| m.content("")
             .embed(|e|
              e.color((255, 20, 7))
               .title("Warcraft III Activity ☥ Status Grid")
               .description(stats_str)
               .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
               .image(weekly.popular_hours)
               .timestamp(now.to_rfc3339())
    )).await?;
  }
  Ok(())
}

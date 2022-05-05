use crate::{
  types::tracking::{ W3CStats, GameMode },
  common::constants::{ W3C_STATS_ROOM, W3C_STATS_MSG },
  steins::warcraft::poller::GAMES,
  commands::w3c::{ generate_popularhours, get_mmm, secs_to_str }
};

use chrono::{
  Timelike,
  Datelike
};
use serenity::prelude::*;
use std::collections::BTreeMap;
use async_std::fs;

const WEEKLY_STATS_FNAME: &str  = "weekly.yml";

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WeeklyWinLoses {
  pub wins: u64,
  pub losses: u64
}

type StatusStats = BTreeMap<String, WeeklyWinLoses>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Daily {
  pub statistics: StatusStats,
  pub statistics2: StatusStats
}

impl Daily {
  fn new() -> Daily {
    Daily {
      statistics: BTreeMap::new(),
      statistics2: BTreeMap::new()
    }
  }
}

type DailyStats = [Daily; 7];

#[derive(Clone, Serialize, Deserialize)]
pub struct Weekly {
  pub reset_day: u32,
  pub stats: DailyStats,
  pub popular_hours: String
}

pub async fn get_weekly(ctx: &Context) -> anyhow::Result<Weekly> {
  if !std::path::Path::new(WEEKLY_STATS_FNAME).exists() {
    clear_weekly(ctx, chrono::Utc::now().date().naive_utc().day()).await?;
  }
  let contents = fs::read_to_string(WEEKLY_STATS_FNAME).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn add_to_weekly(ctx: &Context, p: &str, win: bool, solo: bool) -> anyhow::Result<()> {
  let mut current_weekly = get_weekly(ctx).await?;
  let weekly_stats: &mut StatusStats =
    if solo { &mut current_weekly.stats[0].statistics }
       else { &mut current_weekly.stats[0].statistics2 };
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

async fn clear_weekly(ctx: &Context, day: u32) -> anyhow::Result<()> {
  let poplar_hours =
    if let Some(generated_image) = generate_popularhours(ctx).await? {
      generated_image
    } else {
      "https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png".to_string()
    };
  let init = if !std::path::Path::new(WEEKLY_STATS_FNAME).exists() {
      Weekly {
        reset_day: day,
        stats: [(); 7].map(|_| Daily::new()),
        popular_hours: poplar_hours
      }
    } else {
      let contents = fs::read_to_string(WEEKLY_STATS_FNAME).await?;
      let old: Weekly = serde_yaml::from_str(&contents)?;
      let mut old_stats = old.stats;
      old_stats[..].rotate_right(1);
      old_stats[0].statistics.clear();
      old_stats[0].statistics2.clear();
      Weekly {
        reset_day: day,
        stats: old_stats,
        popular_hours: poplar_hours
      }
    };
  let yml = serde_yaml::to_string(&init)?;
  fs::write(WEEKLY_STATS_FNAME, yml).await?;
  Ok(())
}

pub async fn status_update(ctx: &Context, stats: &W3CStats) -> anyhow::Result<()> {
  if let Ok(mut statusmsg) = W3C_STATS_ROOM.message(ctx, W3C_STATS_MSG).await {
    let weekly = get_weekly(ctx).await?;
    let now = chrono::Utc::now();
    // only check on midnight
    if now.hour() == 0 {
      let now_day = now.date().naive_utc().day();
      if now_day != weekly.reset_day {
        clear_weekly(ctx, now_day).await?;
      }
    }
    let ( (z1, q1)
        , (z2, q2)
        , (z3, q3)
        , searching ) = get_mmm(ctx).await?;
    let (q1s, q2s, q3s) = ( secs_to_str(q1)
                          , secs_to_str(q2)
                          , secs_to_str(q3) );

    let mut tracking_info = vec![];
    let mut tracking_players = vec![];
    { // Games lock scope
      let games_lock = GAMES.lock().await;
      for game in games_lock.values() {
        if game.still_live {
          for fp in game.players.iter() {
            let name = fp.player.battletag
                        .split('#')
                        .collect::<Vec<&str>>()[0];
            let game_mode_str = match game.mode {
              GameMode::Solo  => "1x1",
              GameMode::Team2 => "2x2",
              GameMode::Team4 => "4x4"
            };
            tracking_players.push(fp.player.battletag.clone());
            tracking_info.push(
              format!("{} play {} for {} mins"
              , name
              , game_mode_str
              , game.passed_time)
            );
          }
        }
      }
    }
    let mut searching_info = vec![];
    for (ps, ss) in searching {
      if !tracking_players.iter().any(|tp| tp == &ps) {
        let name = ps.split('#')
                     .collect::<Vec<&str>>()[0];
        searching_info.push(
          format!("{name} {ss}")
        );
      }
    }
    let tracking_str = 
      if tracking_info.is_empty() {
        if searching_info.is_empty() {
          String::from("currently no games")
        } else {
          searching_info.join("\n")
        }
      } else if searching_info.is_empty() {
          tracking_info.join("\n")
      } else {
        format!("{}\n{}"
          , searching_info.join("\n")
          , tracking_info.join("\n")
        )
      };
    let mut weekly_str = vec![];
    let mut weekly_statistics = StatusStats::new();
    let mut weekly_statistics2 = StatusStats::new();
    for stat in  weekly.stats {
      weekly_statistics.extend(stat.statistics);
      weekly_statistics2.extend(stat.statistics2);
    }
    for wss in &[weekly_statistics, weekly_statistics2] {
      let mut ws = Vec::from_iter(wss);
      ws.sort_by( |&(_, a), &(_, b)| (b.wins + b.losses).cmp(&(a.wins + a.losses)) );
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
1x1 {} search for {} GAMES: {}
2x2 {} search for {} GAMES: {}
4x4 {} search for {} GAMES: {}
```
__**currently playing:**__
```
{}
```"
    , weekly_str[0]
    , weekly_str[1]
    , z1, q1s, stats.games_solo
    , z2, q2s, stats.games_2x2
    , z3, q3s, stats.games_4x4
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

use crate::{
  types::tracking::{ W3CStats, GameMode },
  common::{
    constants::{ W3C_STATS_ROOM, W3C_STATS_MSG, W3C_STATS_MSG2 },
    colors::gen_colors
  },
  steins::warcraft::poller::GAMES,
  commands::w3c::{ get_mmm, secs_to_str }
};

use chrono::{
  Timelike,
  Datelike
};
use serenity::{
  prelude::*,
  builder::*,
  model::channel::AttachmentType
};

use std::collections::{
  BTreeMap,
  HashMap
};
use async_std::fs;

use crate::common::constants::APM_PICS;
use plotters::prelude::*;
use stroke::{
  Bezier,
  PointN,
  Point
};

const DAYS_FOR_STATUS: usize = 14;

const WEEKLY_STATS_FNAME: &str  = "weekly.yml";
const BEZIER_STEPS: usize = 1000;

const KURISU_LINK: &str = "https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png";

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DailyWinLoses {
  pub wins: u64,
  pub losses: u64,
  pub mmr: u32
}

type StatusStats = BTreeMap<String, DailyWinLoses>;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Daily {
  pub statistics: StatusStats,
  pub statistics2: StatusStats
}

type DailyStats = [Daily; DAYS_FOR_STATUS];

#[derive(Clone, Serialize, Deserialize)]
pub struct Weekly {
  pub reset_day: u32,
  pub stats: DailyStats,
  pub popular_hours: String,
  pub stats_graph: String,
  pub stats_graph2: String
}

pub async fn get_weekly(ctx: &Context) -> anyhow::Result<Weekly> {
  if !std::path::Path::new(WEEKLY_STATS_FNAME).exists() {
    clear_weekly(ctx, chrono::Utc::now().date().naive_utc().day()).await?;
  }
  let contents = fs::read_to_string(WEEKLY_STATS_FNAME).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn add_to_weekly( ctx: &Context
                          , p: &str
                          , win: bool
                          , xmmr: u32
                          , solo: bool
                          ) -> anyhow::Result<()> {
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
    p_stats.mmr = xmmr;
  } else {
    let stats = DailyWinLoses {
      wins    : if win { 1 } else { 0 },
      losses  : if win { 0 } else { 1 },
      mmr     : xmmr
    };
    weekly_stats.insert(p.to_string(), stats);
  }
  let yml = serde_yaml::to_string(&current_weekly)?;
  fs::write(WEEKLY_STATS_FNAME, yml).await?;
  Ok(())
}

#[allow(clippy::needless_range_loop)]
pub async fn generate_stats_graph( ctx: &Context
                                 , solo: bool
                                 , weeky: &DailyStats ) -> anyhow::Result<Option<String>> {
  let mut weekly_statis_image: Option<String> = None;
  let fname_weekly_statis =
    if solo {
      String::from("weeky_stats.png")
    } else {
      String::from("weeky_stats2.png")
    };
  { // because of Rc < > in BitMapBackend I need own scope here
    let mut plx_vec = vec![];
    let mut min_mmr = 1499;
    let mut max_mmr = 1501;
    let mut stats_vec: HashMap<String, [f64; DAYS_FOR_STATUS]> = HashMap::new();
    for (n, d) in weeky.iter().rev().enumerate() {
      let stats = if solo {
          &d.statistics
        } else {
          &d.statistics2
        };
      for p in stats {
        max_mmr = std::cmp::max(max_mmr, p.1.mmr);
        min_mmr = std::cmp::min(min_mmr, p.1.mmr);
        if let Some(sv) = stats_vec.get_mut(p.0) {
          for i in n..DAYS_FOR_STATUS {
            sv[i] = p.1.mmr as f64;
          }
        } else {
          let dd: [f64; DAYS_FOR_STATUS] = [p.1.mmr as f64; DAYS_FOR_STATUS];
          stats_vec.insert(p.0.clone(), dd);
        }
      }
    }

    let colors = gen_colors(stats_vec.len());
    for (i, (strx, px)) in stats_vec.iter().enumerate() {
      let (red, green, blue) = colors[i];
      let color = RGBColor(red, green, blue);
      let style: ShapeStyle = ShapeStyle::from(color).stroke_width(2);
      let pxx = px.iter().enumerate()
                  .map(|(i, x)| PointN::new([i as f64, *x as f64]))
                  .collect::<Vec<PointN<f64,2>>>();
      let bezier: Bezier<PointN<f64, 2>, DAYS_FOR_STATUS> = Bezier::new( pxx.try_into().unwrap() );
      let mut bezier_graph: Vec<(f64, f64)> = Vec::with_capacity(BEZIER_STEPS);
      for t in 0..BEZIER_STEPS {
        let t = t as f64 * 1f64 / (BEZIER_STEPS as f64);
        let p = bezier.eval(t);
        bezier_graph.push((p.axis(0), p.axis(1)));
      }
      plx_vec.push((style, strx, bezier_graph));
    }

    let root_area = BitMapBackend::new(&fname_weekly_statis, (1000, 500)).into_drawing_area();
    root_area.fill(&RGBColor(47, 49, 54))?;
    let mut cc = ChartBuilder::on(&root_area)
      .margin(5u32)
      .set_all_label_area_size(50u32)
      .build_cartesian_2d(0.0f64..(DAYS_FOR_STATUS - 1) as f64, min_mmr as f64..max_mmr as f64)?;
    cc.configure_mesh()
      .label_style(("monospace", 16).into_font().color(&RGBColor(150, 150, 150)))
      .x_labels(24)
      .y_labels(10)
      .axis_style(&RGBColor(80, 80, 80))
      .draw()?;
    for (st, player_str, plx) in plx_vec {
      cc.draw_series(LineSeries::new(plx, st))?
        .label(player_str.as_str())
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], st));
    }
    cc.configure_series_labels()
      .position(SeriesLabelPosition::LowerRight)
      .border_style(&BLACK)
      .label_font(("monospace", 19).into_font().color(&RGBColor(200, 200, 200)))
      .draw()?;
  }
  match APM_PICS.send_message(&ctx, CreateMessage::new()
    .add_file(AttachmentType::Path(std::path::Path::new(&fname_weekly_statis)))).await {
    Ok(msg) => {
      if !msg.attachments.is_empty() {
        let img_attachment = &msg.attachments[0];
        weekly_statis_image = Some(img_attachment.url.clone());
      }
    },
    Err(why) => {
      error!("Failed to download and post stream img {why}");
    }
  };
  if let Err(why) = fs::remove_file(&fname_weekly_statis).await {
    error!("Error removing popular hours png {why}");
  }
  Ok(weekly_statis_image)
}

pub async fn clear_weekly(ctx: &Context, day: u32) -> anyhow::Result<()> {
  let poplar_hours = KURISU_LINK.to_string();
  let init = if !std::path::Path::new(WEEKLY_STATS_FNAME).exists() {
      Weekly {
        reset_day: day,
        stats: Default::default(),
        popular_hours: poplar_hours,
        stats_graph: KURISU_LINK.to_string(),
        stats_graph2: KURISU_LINK.to_string()
      }
    } else {
      let contents = fs::read_to_string(WEEKLY_STATS_FNAME).await?;
      let old: Weekly = serde_yaml::from_str(&contents)?;
      let mut old_stats = old.stats;
      let weekly_stats_graph =
      if let Ok(Some(wsg)) = generate_stats_graph(ctx, true, &old_stats).await {
          wsg
        } else {
          KURISU_LINK.to_string()
        };
        let weekly_stats_graph2 =
        if let Ok(Some(wsg)) = generate_stats_graph(ctx, false, &old_stats).await {
            wsg
          } else {
            KURISU_LINK.to_string()
            .to_string()
          };
      old_stats[..].rotate_right(1);
      old_stats[0].statistics.clear();
      old_stats[0].statistics2.clear();
      Weekly {
        reset_day: day,
        stats: old_stats,
        popular_hours: poplar_hours,
        stats_graph: weekly_stats_graph,
        stats_graph2: weekly_stats_graph2
      }
    };
  let yml = serde_yaml::to_string(&init)?;
  fs::write(WEEKLY_STATS_FNAME, yml).await?;
  Ok(())
}

fn merge_stats(s1: &mut StatusStats, s2: &StatusStats) {
  for (btag, values) in s2.iter() {
    s1.entry(btag.to_string())
      .and_modify(|existing| {
        existing.wins   += values.wins;
        existing.losses += values.losses;
      }).or_insert(*values);
  }
}

pub async fn status_update(ctx: &Context, stats: &W3CStats) -> anyhow::Result<()> {
  if let Ok(mut statusmsg) = W3C_STATS_ROOM.message(ctx, W3C_STATS_MSG).await {
  if let Ok(mut statusmsg2) = W3C_STATS_ROOM.message(ctx, W3C_STATS_MSG2).await {

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
    for stat in &weekly.stats {
      merge_stats(&mut weekly_statistics, &stat.statistics);
      merge_stats(&mut weekly_statistics2, &stat.statistics2);
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
```
{}
```
"
    , weekly_str[1]);
    statusmsg.edit(ctx, EditMessage::default()
             .embed(CreateEmbed::new()
               .color((255, 20, 7))
               .title( &format!("2x2/4x4 stats for {DAYS_FOR_STATUS} days") )
               .description(stats_str)
               .thumbnail(&weekly.popular_hours)
               .image(&weekly.stats_graph2)
               .timestamp(now)
    )).await?;

  let stats_str2 = format!(
"
```
{}
```
__**currently running:**__
```
1x1 {} search {} GAMES: {}
2x2 {} search {} GAMES: {}
4x4 {} search {} GAMES: {}
```
__**currently playing:**__
```
{}
```"
          , weekly_str[0]
          , z1, q1s, stats.games_solo
          , z2, q2s, stats.games_2x2
          , z3, q3s, stats.games_4x4
          , tracking_str);
          statusmsg2.edit(ctx, EditMessage::default().content("")
                    .embed(CreateEmbed::new()
                      .color((255, 20, 7))
                      .title( &format!("Solo stats for {DAYS_FOR_STATUS} days") )
                      .description(stats_str2)
                      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
                      .image(&weekly.stats_graph)
                     . timestamp(now)
          )).await?;
  }}
  Ok(())
}

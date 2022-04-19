use crate::{
  types::tracking::{ W3CStats, GameMode },
  common::constants::{ W3C_STATS_ROOM, W3C_STATS_MSG },
  commands::w3c::CURRENT_SEASON,
  steins::warcraft::poller::GAMES
};

use serenity::prelude::*;

use std::sync::atomic::Ordering::Relaxed;

pub async fn status_update(ctx: &Context, stats: &W3CStats) -> anyhow::Result<()> {
  if let Ok(mut statusmsg) = W3C_STATS_ROOM.message(ctx, W3C_STATS_MSG).await {
    let season = CURRENT_SEASON.load(Relaxed);

    let mut tracking_info = vec![];
    { // Games lock scope
      let games_lock = GAMES.lock().await;
      for game in games_lock.values() {
        if let Some(fp) = game.players.first() {
          let game_mode_str = match game.mode {
            GameMode::Solo  => "1x1",
            GameMode::Team2 => "2x2",
            GameMode::Team4 => "4x4"
          };
          tracking_info.push(
            format!("{} playing {} game for {} minutes"
            , fp.player.battletag
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
    let stats_str = format!(
"
__**currently running:**__
```
SOLO GAMES: {}
2x2  GAMES: {}
4x4  GAMES: {}
```

__**currently playing:**__
```
{}
```

__**meta info:**__
```
current season: {}
```"
    , stats.games_solo
    , stats.games_2x2
    , stats.games_4x4
    , tracking_str
    , season);
    statusmsg.edit(ctx, |m| m.content("")
             .embed(|e|
              e.color((32, 32, 32))
               .title("Status Grid")
               .description(stats_str)
               .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
               .image("https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
               .timestamp(chrono::Utc::now().to_rfc3339())
    )).await?;
  }
  Ok(())
}

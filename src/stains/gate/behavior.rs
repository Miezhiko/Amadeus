use crate::{
  types::options::IOptions,
  stains::{
    gate::social::activate_social_skils,
    gate::tracking::{ team_games::activate_games_tracking
                    , streamers::activate_streamers_tracking },
    ai::chain
  },
  commands::pad::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    id::GuildId,
    gateway::Activity
  }
};

use chrono::{ Utc, DateTime };
use tokio::sync::Mutex;

lazy_static! {
  pub static ref START_TIME: Mutex<DateTime<Utc>>  = Mutex::new(Utc::now());
}

pub async fn activate(ctx: &Context, options: &IOptions) {
  info!("activation has started");
  let loading = format!("Loading {}", env!("CARGO_PKG_VERSION").to_string());
  ctx.set_activity(Activity::listening(loading.as_str())).await;
  ctx.idle().await;

  lazy_static::initialize(&START_TIME);

  // set actual season for pad statistics
  update_current_season().await;

  if options.guild != 0 {
    let guild_id = GuildId( options.guild );

    // updating ai:chain cache
    chain::update_cache(&ctx, &guild_id).await;

    if let Ok(channels) = guild_id.channels(ctx).await {

      activate_social_skils(
        ctx, &channels
           , guild_id).await;
      activate_streamers_tracking(
        ctx, &channels
           , options
           ).await;
      activate_games_tracking(
        ctx, &channels
           , options
           ).await;

      let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
      ctx.set_activity(Activity::playing(version.as_str())).await;
      ctx.online().await;
    }
  }
}

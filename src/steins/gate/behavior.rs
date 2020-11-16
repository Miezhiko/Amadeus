use crate::{
  types::options::IOptions,
  common::options,
  steins::{
    gate::social::activate_social_skils,
    gate::tracking::{ team_games::activate_games_tracking
                    , streamers::activate_streamers_tracking
                    , w3info::activate_w3info_tracking },
    ai::chain
  },
  commands::pad::update_current_season,
  commands::owner::twitch_update
};

use serenity::{
  prelude::*,
  model::{
    id::{ GuildId, ChannelId, UserId },
    channel::GuildChannel
  }
};

use std::collections::HashMap;

use chrono::{ Utc, DateTime };
use tokio::sync::Mutex;

lazy_static! {
  pub static ref START_TIME: Mutex<DateTime<Utc>>  = Mutex::new(Utc::now());
}

pub async fn activate(ctx: Context, options: &IOptions, amadeus: &UserId) {
  info!("activation has started");

  lazy_static::initialize(&START_TIME);

  // set actual season for pad statistics
  update_current_season(&ctx).await;

  // generate new twitch toke
  if let Err(why) = twitch_update(&ctx).await {
    error!("Twitch token update failed {:?}", why);
  }

  info!("loading Kathoey");
  lazy_static::initialize(&chain::KATHOEY);

  info!("starting background threads");
  // Now there are several lists of channels and several Guilds
  let servers = options.servers.iter()
                               .map(|srv| GuildId(srv.id))
                               .collect::<Vec<GuildId>>();
  let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
  for server in servers {
    if let Ok(serv_channels) = server.channels(&ctx).await {
      all_channels.extend(serv_channels);
    }
  }

  // updating ai:chain cache
  chain::update_cache(&ctx, &all_channels).await;

  let ac = std::sync::Arc::new(ctx);
  activate_social_skils(&ac).await;

  let opts = options::get_roptions().await.unwrap();
  let access_token = opts.twitch;

  activate_streamers_tracking(
    &ac, options, access_token.clone()
       ).await;
  activate_games_tracking(
    &ac, options, access_token
       , amadeus.0
       ).await;
  activate_w3info_tracking(&ac).await;
}

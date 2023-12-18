use crate::{
  types::options::IOptions,
  common::{ options, system },
  steins::{
    gate::START_TIME,
    gate::tracking::{ system::activate_system_tracker
                    , team_games::activate_games_tracking
                    , streamers::activate_streamers_tracking
                    , w3info::activate_w3info_tracking
                    , dev::activate_dev_tracker }
  },
  commands::w3c::update_current_season
};

use crate::{
  salieri,
  steins::{ ai::cache
          , gate::tracking::social::activate_social_skils }
};

use serenity::{
  prelude::*,
  model::{
    application::Command,
    id::{ GuildId, ChannelId, UserId },
    channel::GuildChannel
  }
};

use std::collections::HashMap;
use once_cell::sync::Lazy;

pub async fn activate(ctx: Context, options: &IOptions, amadeus: &UserId) {
  info!("activation has started");

  Lazy::force(&START_TIME);

  // clean up global application commands
  if let Err(why) = Command::set_global_commands(&ctx.http, vec![]).await {
    error!("Failed to clean global application commands, {why}");
  }

  // set actual season for pad statistics
  update_current_season(&ctx).await;

  // generate new twitch toke
  info!("updating twitch token");
  if let Err(why) = system::hacks::twitch_update(&ctx).await {
    error!("Twitch token update failed, {why}");
  }

  info!("loading Kathoey");
  Lazy::force(&cache::KATHOEY);

  info!("starting background threads");
  if options.gencache_on_start {
    // Now there are several lists of channels and several Guilds
    let servers = options.servers.iter()
                                .map(|srv| GuildId::new(srv.id))
                                .collect::<Vec<GuildId>>();
    let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
    for server in &servers {
      if let Ok(serv_channels) = server.channels(&ctx).await {
        all_channels.extend(serv_channels);
      }
    }
    let home = GuildId::new(options.guild);
    if let Ok(serv_channels) = home.channels(&ctx).await {
      all_channels.extend(serv_channels);
    }

    // updating ai:chain cache
    cache::update_cache(&ctx, &all_channels).await;
  }

  let ac = std::sync::Arc::new(ctx);
  let oc = std::sync::Arc::new(options.clone());

  #[cfg(feature = "salieri")]
  {
    info!("connecting to Salieri");
    if let Err(why) = salieri::salieri_init(&ac).await {
      error!("failed to init Salieri services {why}");
    }
  }

  #[cfg(feature = "naoko")]
  {
    info!("connecting to Naoko");
    if let Err(why) = naoko::naoko_init(&ac).await {
      error!("failed to init Naoko services {why}");
    }
  }

  activate_system_tracker(&ac).await;
  activate_social_skils(&ac).await;

  let opts = options::get_roptions().await.unwrap();
  let access_token = opts.twitch;

  activate_streamers_tracking(
    &ac, &oc, access_token.clone()
       ).await;
  activate_games_tracking(
    &ac, &oc, access_token
       , amadeus.get()
       ).await;
  activate_w3info_tracking(&ac).await;
  activate_dev_tracker(&ac, &options.github_auth).await;
}

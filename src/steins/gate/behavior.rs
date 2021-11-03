use crate::{
  types::options::IOptions,
  common::{ options, system },
  steins::{
    gate::START_TIME,
    gate::tracking::{ system::activate_system_tracker
                    , team_games::activate_games_tracking
                    , social::activate_social_skils
                    , streamers::activate_streamers_tracking
                    , w3info::activate_w3info_tracking },
    ai::cache
  },
  commands::w3c::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    interactions::application_command::ApplicationCommand,
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
  if let Err(why) = ApplicationCommand::set_global_application_commands(&ctx.http, |cs| cs
  ).await {
    error!("Failed to clean global application commands {}", why);
  }

  // set actual season for pad statistics
  update_current_season(&ctx).await;

  // generate new twitch toke
  if let Err(why) = system::hacks::twitch_update(&ctx).await {
    error!("Twitch token update failed {:?}", why);
  }

  info!("loading Kathoey");
  Lazy::force(&cache::KATHOEY);

  info!("starting background threads");
  // Now there are several lists of channels and several Guilds
  let servers = options.servers.iter()
                               .map(|srv| GuildId(srv.id))
                               .collect::<Vec<GuildId>>();
  let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
  for server in &servers {
    if let Ok(serv_channels) = server.channels(&ctx).await {
      all_channels.extend(serv_channels);
    }
  }
  let home = GuildId(options.guild);
  if let Ok(serv_channels) = home.channels(&ctx).await {
    all_channels.extend(serv_channels);
  }

  // updating ai:chain cache
  cache::update_cache(&ctx, &all_channels, options.lazy_static_models).await;

  let ac = std::sync::Arc::new(ctx);
  activate_system_tracker(&ac).await;
  activate_social_skils(&ac).await;

  let opts = options::get_roptions().await.unwrap();
  let access_token = opts.twitch;

  activate_streamers_tracking(
    &ac, options, access_token.clone()
       , servers
       ).await;
  activate_games_tracking(
    &ac, options, access_token
       , amadeus.0
       ).await;
  activate_w3info_tracking(&ac).await;
}

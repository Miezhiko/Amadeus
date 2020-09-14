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
  commands::pad::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    id::{ GuildId, ChannelId },
    channel::GuildChannel
  }
};

use std::collections::HashMap;

use chrono::{ Utc, DateTime };
use tokio::sync::Mutex;

lazy_static! {
  pub static ref START_TIME: Mutex<DateTime<Utc>>  = Mutex::new(Utc::now());
}

pub async fn activate(ctx: &Context, options: &IOptions) {
  info!("activation has started");

  lazy_static::initialize(&START_TIME);

  // set actual season for pad statistics
  update_current_season().await;

  // Now there are several lists of channels and several Guilds
  if options.guild != 0 {
    let hemo_guild_id = GuildId( options.guild );
    let servers = options.servers.iter()
                                 .map(|srv| GuildId(srv.id))
                                 .collect::<Vec<GuildId>>();

    let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();

     for server in servers {
      if let Ok(serv_channels) = server.channels(ctx).await {
        all_channels.extend(serv_channels);
      }
    }

    let hemo_channels: HashMap<ChannelId, GuildChannel> =
      if let Ok(channels) = hemo_guild_id.channels(ctx).await {
        channels
      } else {
        HashMap::new()
      };

    // updating ai:chain cache
    chain::update_cache(&ctx, &all_channels).await;

    if !hemo_channels.is_empty() {
      activate_social_skils(
        ctx, &all_channels).await;

      let opts = options::get_roptions().await.unwrap();
      let access_token = opts.twitch;

      activate_streamers_tracking(
        ctx, &hemo_channels
           , options, access_token.clone()
           ).await;
      activate_games_tracking(
        ctx, &hemo_channels
           , options, access_token
           ).await;
      activate_w3info_tracking(
        ctx, &all_channels
           ).await;
    }
  }
}

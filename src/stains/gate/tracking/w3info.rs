use crate::{
  commands::warcraft,
  common::help::channel::channel_by_name
};

use serenity::{
  prelude::*,
  model::{
    id::ChannelId,
    channel::GuildChannel
  }
};

use std::{
  time,
  collections::HashMap
};

use chrono::{ Duration, DateTime, Utc };

pub async fn activate_w3info_tracking(
                     ctx:       &Context
                   , channels:  &HashMap<ChannelId, GuildChannel> ) {

  let mut channels_to_process = vec![];

  if let Some((log_channel, _)) = channel_by_name(&ctx, &channels, "❗fake-news").await {
    channels_to_process.push(log_channel);
  }
  if let Some((zaryanka_channel, _)) = channel_by_name(&ctx, &channels, "🏰турниры🏰").await {
    channels_to_process.push(zaryanka_channel);
  }

  if !channels_to_process.is_empty() {
    let today : DateTime<Utc> = Utc::now();

    let ctx_clone = ctx.clone();
    let thread_channels = channels_to_process.iter()
                                             .map(|ch| **ch)
                                             .collect::<Vec<ChannelId>>();
    tokio::spawn(async move {
      loop {
        for chan in &thread_channels {
          if let Err(why) =
            warcraft::tour_internal( &ctx_clone
                                   , &chan, today
                                   , false, false
                                   ).await {
            error!("Failed to post today tour events {:?}", why);
          }
          if let Err(why) =
            warcraft::tour_internal( &ctx_clone
                                   , &chan, today + Duration::days(1)
                                   , false, false
                                   ).await {
            error!("Failed to post tomorrow tour events {:?}", why);
          }
        }
        // check every day
        tokio::time::delay_for(time::Duration::from_secs(60*60*24)).await;
      }
    });
  }
}

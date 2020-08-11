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

use chrono::{ DateTime, Utc };

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

    // Delete today news, today news will be reposted
    let today : DateTime<Utc> = Utc::now();
    let today_date = today.date();
    for shannel in &channels_to_process {
      if let Ok(vec_msg) = shannel.messages(&ctx, |g| g.limit(5)).await {
        let mut vec_id = Vec::new();
        for message in vec_msg {
          if message.timestamp.date() == today_date
          && message.is_own(ctx).await {
            vec_id.push(message.id);
          }
        }
        if !vec_id.is_empty() {
          match shannel.delete_messages(&ctx, vec_id.as_slice()).await {
            Ok(nothing)  => nothing,
            Err(err) => warn!("Failed to clean live messages {}", err),
          };
        }
      }
    }

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
                                   , true, false
                                   ).await {
            error!("Failed to post tour events {:?}", why);
          }
        }
        // check every day
        tokio::time::delay_for(time::Duration::from_secs(60*60*24)).await;
      }
    });
  }
}

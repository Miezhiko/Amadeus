use crate::common::msg::{ split_code, split_message, MESSAGE_LIMIT };

use serenity::{
  builder::CreateMessage,
  model::{ id::GuildId, id::ChannelId, channel::GuildChannel },
  prelude::*
};

use futures_util::stream::{self, StreamExt};

#[allow(dead_code)]
async fn log_any<F> ( ctx: &Context
                  , guild_id: &GuildId
                  , f: F)
    where for <'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
  if let Ok(channels) = guild_id.channels(ctx).await {
    let log_channels = stream::iter(channels.iter())
    .filter_map(|(c, _)| async move {
      if let Some(name) = c.name(&ctx).await {
        if name == "log" { Some(c) } else { None }
      } else { None }
    }).collect::<Vec<&ChannelId>>().await;
    if log_channels.len() > 0 {
      let channel = log_channels[0];
      if let Err(why) = channel.send_message(ctx, f).await {
        error!("Failed to log new user {:?}", why);
      }
    }
  }
}

async fn serenity_channel_message_single(ctx: &Context, chan : &GuildChannel, text: &str) {
  if let Err(why) = chan.say(ctx, text).await {
    error!("Error sending log message: {:?}", why);
  }
}
async fn serenity_channel_message_multi(ctx: &Context, chan : &GuildChannel, texts : Vec<&str>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, text).await;
  }
}
async fn serenity_channel_message_multi2(ctx: &Context, chan : &GuildChannel, texts : Vec<String>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, text.as_str()).await;
  }
}
async fn channel_message(ctx: &Context, chan : &GuildChannel, text: &str) {
  if text.len() > MESSAGE_LIMIT {
    if text.starts_with("```") {
      serenity_channel_message_multi2(ctx, chan, split_code(text)).await;
    } else {
      serenity_channel_message_multi(ctx, chan, split_message(text)).await;
    }
  } else {
    serenity_channel_message_single(ctx, chan, text).await;
  }
}

pub async fn log(ctx: &Context, guild_id: &GuildId, text: &str) {
  if let Ok(channels) = guild_id.channels(ctx).await {
    let log_channels = stream::iter(channels.iter())
    .filter_map(|(c, gc)| async move {
      if let Some(name) = c.name(&ctx).await {
        if name == "log" { Some(gc) } else { None }
      } else { None }
    }).collect::<Vec<&GuildChannel>>().await;
    if log_channels.len() > 0 {
      let channel = log_channels[0];
      channel_message(ctx, channel, text).await;
    }
  }
}

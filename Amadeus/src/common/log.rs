use crate::common::{ help::channel::channel_by_name
                   , msg::{ split_code, split_message }
                   };

use serenity::{
  model::{ id::GuildId
         , channel::{ Message, GuildChannel }
         },
  prelude::*
};

async fn serenity_channel_message_single(ctx: &Context, chan: &GuildChannel, text: &str) {
  if let Err(why) = chan.say(ctx, text).await {
    error!("Error sending log message: {why}");
  }
}
async fn serenity_channel_message_multi(ctx: &Context, chan: &GuildChannel, texts: Vec<&str>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, text).await;
  }
}
async fn serenity_channel_message_multi2(ctx: &Context, chan: &GuildChannel, texts: Vec<String>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, &text).await;
  }
}
async fn channel_message(ctx: &Context, chan: &GuildChannel, text: &str) {
  if Message::overflow_length(text).is_some() {
    if text.starts_with("```") {
      serenity_channel_message_multi2(ctx, chan, split_code(text)).await;
    } else {
      serenity_channel_message_multi(ctx, chan, split_message(text)).await;
    }
  } else {
    serenity_channel_message_single(ctx, chan, text).await;
  }
}

#[allow(dead_code)]
pub async fn log(ctx: &Context, guild_id: &GuildId, text: &str) {
  if let Ok(channels) = guild_id.channels(ctx).await {
    if let Some((_, guild_channel)) = channel_by_name(ctx, &channels, "log").await {
      channel_message(ctx, guild_channel, text).await;
    }
  }
}

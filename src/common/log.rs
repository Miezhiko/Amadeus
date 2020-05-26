use crate::common::msg::{ split_code, split_message, MESSAGE_LIMIT };

use serenity::{
  builder::CreateMessage,
  model::{ id::GuildId, channel::GuildChannel },
  prelude::*
};

pub fn log_any<F> ( ctx: &Context
                  , guild_id: &GuildId
                  , f: F)
    where for <'a, 'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a> {
  if let Ok(channels) = guild_id.channels(ctx) {
    let log_channel = channels.iter().find(|&(c, _)|
      if let Some(name) = c.name(ctx) {
        name == "log"
      } else {
        false
      });
    if let Some((_, channel)) = log_channel {
      if let Err(why) = channel.send_message(ctx, f) {
        error!("Failed to log new user {:?}", why);
      }
    }
  }
}

fn serenity_channel_message_single(ctx: &Context, chan : &GuildChannel, text: &str) {
  if let Err(why) = chan.say(ctx, text) {
    error!("Error sending log message: {:?}", why);
  }
}
fn serenity_channel_message_multi(ctx: &Context, chan : &GuildChannel, texts : Vec<&str>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, text);
  }
}
fn serenity_channel_message_multi2(ctx: &Context, chan : &GuildChannel, texts : Vec<String>) {
  for text in texts {
    serenity_channel_message_single(ctx, chan, text.as_str());
  }
}
fn channel_message(ctx: &Context, chan : &GuildChannel, text: &str) {
  if text.len() > MESSAGE_LIMIT {
    if text.starts_with("```") {
      serenity_channel_message_multi2(ctx, chan, split_code(text));
    } else {
      serenity_channel_message_multi(ctx, chan, split_message(text));
    }
  } else {
    serenity_channel_message_single(ctx, chan, text);
  }
}

pub fn log(ctx: &Context, guild_id: &GuildId, text: &str) {
  if let Ok(channels) = guild_id.channels(ctx) {
    let log_channel = channels.iter().find(|&(c, _)|
      if let Some(name) = c.name(ctx) {
        name == "log"
      } else {
        false
      });
    if let Some((_, channel)) = log_channel {
      channel_message(ctx, channel, text);
    }
  }
}

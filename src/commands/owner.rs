use crate::{
  common::{
    msg::{ channel_message }
  },
  stains::gate,
  stains::ai::chain::ACTIVITY_LEVEL
};

use serenity::{
  model::{ id::ChannelId
         , channel::* },
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use std::sync::atomic::{ Ordering };

#[command]
pub fn set(ctx: &mut Context, msg: &Message, mut args : Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  if let Ok(property) = args.single::<String>() {
    match property.as_str() {
      "activity" =>
        if let Ok(level) = args.single::<u32>() {
          ACTIVITY_LEVEL.store(level, Ordering::Relaxed);
          let chan_msg = format!("Activity level is: {} now", level);
          channel_message(&ctx, &msg, chan_msg.as_str());
        },
      _ => ()
    }
  }
  Ok(())
}

#[command]
pub fn say(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  let last_channel_u64 = gate::LAST_CHANNEL.load(Ordering::Relaxed);
  if last_channel_u64 != 0 {
    let last_channel_conf = ChannelId( last_channel_u64 );
    if msg.guild_id.is_some() {
      let text = args.message();
      if !text.is_empty() {
        if let Err(why) = last_channel_conf.say(&ctx, text) {
          error!("Failed say {:?}", why);
        }
      }
    }
  }
  Ok(())
}

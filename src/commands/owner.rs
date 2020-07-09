use crate::{
  common::{
    msg::{ channel_message, direct_message }
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
async fn set(ctx: &Context, msg: &Message, mut args : Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  if let Ok(property) = args.single::<String>() {
    match property.as_str() {
      "activity" =>
        if let Ok(level) = args.single::<u32>() {
          ACTIVITY_LEVEL.store(level, Ordering::Relaxed);
          let chan_msg = format!("Activity level is: {} now", level);
          channel_message(&ctx, &msg, chan_msg.as_str()).await;
        },
      _ => ()
    }
  }
  Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message, args : Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let last_channel_u64 = gate::LAST_CHANNEL.load(Ordering::Relaxed);
  if last_channel_u64 != 0 {
    let last_channel_conf = ChannelId( last_channel_u64 );
    if msg.guild_id.is_some() {
      let text = args.message();
      if !text.is_empty() {
        if let Err(why) = last_channel_conf.say(ctx, text).await {
          error!("Failed say {:?}", why);
        }
      }
    }
  }
  Ok(())
}

#[command]
async fn clear(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if args.len() == 1 {
    let countdown: u64 = args.find().unwrap_or_default();
    for vec in msg.channel_id.messages(ctx, |g| g.before(msg.id).limit(countdown)).await {
      let mut vec_id = Vec::new();
      for message in vec {
        vec_id.push(message.id);
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, &msg, &format!("Deleted {} messages", countdown)).await;
  } else if args.len() == 2 {
    let countdown: u64 = args.find().unwrap_or_default();
    let counter: u64 = args.find().unwrap_or_default();
    let full = countdown + counter;
    for vec in msg.channel_id.messages(ctx, |g| g.before(msg.id).limit(full)).await {
      let mut vec_id = Vec::new();
      let mut i = 0;
      for message in vec.iter().rev() {
        if i < countdown {
          vec_id.push(message.id);
        }
        i += 1;
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, &msg, &format!("Deleted {} messages", countdown)).await;
  }
  Ok(())
}

use crate::common::{ points, msg::channel_message };

use serenity::{
  model::{ channel::*, gateway::Activity },
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

#[command]
async fn idle(ctx: &Context, msg: &Message, args : Args) -> CommandResult {
  let what = args.message();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  ctx.set_activity(Activity::playing(&what)).await;
  ctx.idle().await;
  Ok(())
}

#[command]
async fn stream(ctx: &Context, msg: &Message, mut args : Args) -> CommandResult {
  if let Ok(stream_url) = args.single::<String>() {
    let name = args.single::<String>().unwrap_or_else(|_| "Amadeus".to_string());
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    ctx.set_activity(Activity::streaming(&name, &stream_url)).await;
    ctx.dnd().await;
  }
  Ok(())
}

#[command]
async fn give_win(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild) = msg.guild(&ctx).await {
    if msg.mentions.is_empty() {
      channel_message(ctx, msg, "you need to target points reciever").await;
    } else {
      let target_user = &msg.mentions[0];
      let s = points::add_win_points( *guild.id.as_u64()
                                    , *target_user.id.as_u64()
                                    ).await;
      let out = format!("win registered, {} wins in a row", s);
      channel_message(ctx, msg, out.as_str()).await;
    };
  }
  Ok(())
}

#[command]
async fn register_lose(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild) = msg.guild(&ctx).await {
    if msg.mentions.is_empty() {
      channel_message(ctx, msg, "you need to target points reciever").await;
    } else {
      let target_user = &msg.mentions[0];
      let _ = points::break_streak( *guild.id.as_u64()
                                  , *target_user.id.as_u64()
                                  ).await;
    };
  }
  Ok(())
}

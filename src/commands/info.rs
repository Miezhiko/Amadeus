use crate::{
  common::{
    msg::channel_message,
    db::sled_info
  }
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

#[command]
#[min_args(2)]
async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let key = &args.single::<String>()?;
  let value = args.rest();
  if let Err(why) = sled_info::store(key, value) {
    error!("Failed to register {key}, {why}");
  } else {
    channel_message(ctx, msg, &format!("Registered {key}")).await;
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn show(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let key = &args.single::<String>()?;
  match sled_info::read(key) {
    Ok(val) => {
      channel_message(ctx, msg, &val).await;
    }, Err(why) => {
      error!("Failed to get {key}, {why}");
    }
  }
  Ok(())
}

#[command]
async fn list(ctx: &Context, msg: &Message) -> CommandResult {
  match sled_info::list() {
    Ok(val) => {
      channel_message(ctx, msg, &val).await;
    }, Err(why) => {
      error!("Failed to show info list, {why}");
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn delete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let key = &args.single::<String>()?;
  match sled_info::delete(key) {
    Ok(_) => {
      channel_message(ctx, msg, &format!("Deleted {key}")).await;
    }, Err(why) => {
      error!("Failed to get {key}, {why}");
    }
  }
  Ok(())
}

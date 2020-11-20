use crate::{
  common::{
    msg::channel_message,
    sled
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
  //if let Err(why) = msg.delete(ctx).await {
  //  error!("Error deleting original command {:?}", why);
  //}
  let key = &args.single::<String>()?;
  let value = args.rest();
  if let Err(why) = sled::store(key, value) {
    error!("Failed to register {}, {:?}", key, why);
  } else {
    channel_message(&ctx, &msg, &format!("Registered {}", key)).await;
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn show(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  //if let Err(why) = msg.delete(ctx).await {
  //  error!("Error deleting original command {:?}", why);
  //}
  let key = &args.single::<String>()?;
  match sled::read(key) {
    Ok(val) => {
      channel_message(&ctx, &msg, &val).await;
    }, Err(why) => {
      error!("Failed to get {}, {:?}", key, why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn delete(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  //if let Err(why) = msg.delete(ctx).await {
  //  error!("Error deleting original command {:?}", why);
  //}
  let key = &args.single::<String>()?;
  match sled::delete(key) {
    Ok(_) => {
      channel_message(&ctx, &msg, &format!("Deleted {}", key)).await;
    }, Err(why) => {
      error!("Failed to get {}, {:?}", key, why);
    }
  }
  Ok(())
}

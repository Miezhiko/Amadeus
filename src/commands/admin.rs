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
    let name = args.single::<String>().unwrap_or(String::from("Amadeus"));
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    ctx.set_activity(Activity::streaming(&name, &stream_url)).await;
    ctx.dnd().await;
  }
  Ok(())
}

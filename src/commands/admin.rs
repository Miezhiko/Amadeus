use serenity::{
  model::{ channel::*
         , gateway::Activity },
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

#[command]
pub fn idle(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  let what = args.message();
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  ctx.set_activity(Activity::playing(&what));
  ctx.idle();
  Ok(())
}

#[command]
pub fn stream(ctx: &mut Context, msg: &Message, mut args : Args) -> CommandResult {
  if let Ok(stream_url) = args.single::<String>() {
    let name = args.single::<String>().unwrap_or(String::from("Amadeus"));
    if let Err(why) = msg.delete(&ctx) {
      error!("Error deleting original command {:?}", why);
    }
    ctx.set_activity(Activity::streaming(&name, &stream_url));
    ctx.dnd();
  }
  Ok(())
}

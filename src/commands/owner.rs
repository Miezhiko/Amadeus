use crate::{
  conf
};

use serenity::{
  model::{ id::GuildId, id::ChannelId
         , channel::*
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
pub fn stream(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  let what = args.message();
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  ctx.set_activity(Activity::streaming("Amadeus", &what));
  ctx.dnd();
  Ok(())
}

#[command]
pub fn say(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  //TODO: actually channel is enough
  let conf = conf::parse_config();
  set!{ last_guild_u64 = conf.last_guild_chat.parse::<u64>().unwrap_or(0)
      , last_channel_u64 = conf.last_channel_chat.parse::<u64>().unwrap_or(0) };
  if last_guild_u64 != 0 && last_channel_u64 != 0 {
    set!{ _last_guild_conf = GuildId( last_guild_u64 )
        , last_channel_conf = ChannelId( last_channel_u64 ) };
    if msg.guild_id.is_some() {
      //TODO
      /*if last_guild_conf == msg.guild_id.unwrap() {*/
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

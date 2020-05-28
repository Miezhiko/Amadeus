use crate::{
  conf
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

#[command]
pub fn say(ctx: &mut Context, msg: &Message, args : Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  let conf = conf::parse_config();
  let last_channel_u64 = conf.last_channel_chat.parse::<u64>().unwrap_or(0);
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

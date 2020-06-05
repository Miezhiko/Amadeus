use crate::{
  common::{
    msg::{ channel_message }
  },
  stains::ai::chain
};

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult,
    macros::command
  },
};

#[command]
pub fn quote(ctx: &mut Context, msg: &Message) -> CommandResult {
  if msg.mentions.len() > 0 {
    let target = &msg.mentions[0];
    if let Some(q) = chain::make_quote(ctx, msg, target.id, 9000) {
      let out = format!("Quote from {}: {}", target.name, q);
      channel_message(ctx, msg, out.as_str());
    } else {
      let out = format!("No idea about {}", target.name);
      channel_message(ctx, msg, out.as_str());
    }
  }
  Ok(())
}

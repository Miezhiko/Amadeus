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
  if let Err(why) = msg.delete(&ctx) {
    error!("Error deleting original command {:?}", why);
  }
  if msg.mentions.len() > 0 {
    let target = &msg.mentions[0];
    if let Some(q) = chain::make_quote(ctx, msg, target.id, 9000) {
      let footer = format!("Requested by {}", msg.author.name);
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
      .embed(|e| e
        .author(|a| a.icon_url(&target.face()).name(&target.name))
        .description(q)
        .footer(|f| f.text(footer))
      )) {
        error!("Failed to quote {}, {:?}", target.name, why);
      }
    } else {
      let out = format!("No idea about {}", target.name);
      channel_message(ctx, msg, out.as_str());
    }
  }
  Ok(())
}

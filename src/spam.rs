use crate::collections::team::DISCORDS;

use serenity::{
  prelude::*,
  model::{ id::GuildId
         , channel::Message,
         }
};

pub async fn spam_check(
      guild_id: &GuildId
    , ctx: &Context
    , msg: &Message) {
  if msg.content.contains("disocrds.gift") {
    if let Err(why) = &msg.delete(&ctx).await {
      error!("Error deleting spam {:?}", why);
    }
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
        let timeout = chrono::Utc::now() + chrono::Duration::days(1);
        if let Err(why) = member.disable_communication_until_datetime(ctx, timeout).await {
          error!("Failed to timeout user for a day {why}");
        }
      }
    }
    if let Some(ds) = DISCORDS.get(&guild_id.0) {
      if let Some(log) = ds.log {
        if let Err(why) = log.send_message(&ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
              .title("SCAM MESSAGE BLOCKED")
              .timestamp(chrono::Utc::now().to_rfc3339())
            })).await {
          error!("Failed to log leaving user {why}");
        }
      }
    }
  }
}

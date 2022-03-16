use crate::collections::team::DISCORDS;

use serenity::{
  prelude::*,
  model::{ id::GuildId
         , channel::Message,
         }
};

use regex::Regex;
use once_cell::sync::Lazy;

//TODO: russian slurs
static SLUR: Lazy<Regex> =
  Lazy::new(||
    Regex::new(r"(fag(g|got|tard)?\b|cock\s?sucker(s|ing)?|ni((g{2,}|q)+|[gq]{2,})[e3r]+(s|z)?|bitch(es|ing|y)?|whor(es?|ing)|retard(ed)?s?)").unwrap());

async fn delete( guild_id: &GuildId
               , ctx: &Context
               , msg: &Message
               , disable_communication: bool
               , reason: &str
               ) {
  if let Err(why) = &msg.delete(&ctx).await {
    error!("Error deleting spam {:?}", why);
  }
  if disable_communication {
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
        let timeout = chrono::Utc::now() + chrono::Duration::days(1);
        if let Err(why) = member.disable_communication_until_datetime(ctx, timeout).await {
          error!("Failed to timeout user for a day {why}");
        }
      }
    }
  }
  if let Some(ds) = DISCORDS.get(&guild_id.0) {
    if let Some(log) = ds.log {
      if let Err(why) = log.send_message(&ctx, |m| m
        .embed(|e| {
          e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
           .title(reason)
           .description(&format!( "User UID: {}\n original message: {}"
                                , msg.author.id.0, &msg.content))
           .timestamp(chrono::Utc::now().to_rfc3339())
        })).await {
        error!("Failed to log leaving user {why}");
      }
    }
  }
}

pub async fn spam_check(
      guild_id: &GuildId
    , ctx: &Context
    , msg: &Message) {
  if msg.content.contains("disocrds.gift") {
    delete( guild_id, ctx, msg, true
          , "SCAM MESSAGE BLOCKED" ).await;
  }
  for embed in &msg.embeds {
    if let Some(url) = &embed.url {
      if url.contains("disocrds.gift") {
        delete( guild_id, ctx, msg, true
              , "SCAM MESSAGE BLOCKED" ).await;
      }
    }
  }
  if SLUR.is_match(&msg.content) {
    delete( guild_id, ctx, msg, false
          , "SLUR USED" ).await;
  }
}

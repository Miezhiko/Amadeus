use crate::collections::team::DISCORDS;

use serenity::{
  prelude::*,
  model::{ id::GuildId
         , channel::Message
         , Timestamp
         }
};

use std::{
  collections::HashSet,
  fs
};

use regex::Regex;
use once_cell::sync::Lazy;

static SLUR: Lazy<Regex> =
  Lazy::new(||
    Regex::new(
      r"(fag(g|got|tard)?\b|cunt|cock\s?sucker(s|ing)?|ni((g{2,}|q)+|[gq]{2,})[ae3r]+(s|z)?s?)"
    ).unwrap());

static LIGHT_SLUR_LOL: Lazy<Regex> =
    Lazy::new(||
      Regex::new(
        r"(bitch(es|ing|y)|whor(es?|ing)|bastard)"
      ).unwrap());

// https://raw.githubusercontent.com/nikolaischunk/discord-phishing-links/main/domain-list.json
const BAN_LIST_FILE_NAME: &str = "ban-list.json";

const LINK_SEPARATORS: &[char] = &[' ',',',':','\n','\t','<','>','|','"','\'','@'];

#[derive(serde::Deserialize, Debug)]
struct BanList {
  domains: HashSet<String>
}

fn get_banlist() -> anyhow::Result<BanList> {
  let contents = fs::read_to_string(BAN_LIST_FILE_NAME)?;
  serde_json::from_str(&contents).map_err(|e| anyhow!("Failed to parse ban list {e}"))
}

static BAN_LIST: Lazy<BanList> = Lazy::new(|| get_banlist().unwrap() );

async fn delete( guild_id: &GuildId
               , ctx: &Context
               , msg: &Message
               , disable_communication: bool
               , really_delete: bool
               , reason: &str
               ) {
  if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
    if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
      if disable_communication {
        let timeout = chrono::Utc::now() + chrono::Duration::days(1);
        if let Err(why) = member.disable_communication_until_datetime(ctx, Timestamp::from( timeout) ).await {
          error!("Failed to timeout user for a day {why}");
        }
      } else if let Ok(permissions) = member.permissions(&ctx) {
        if permissions.ban_members() {
          if !really_delete {
            // completely ignore harmful words
            return;
          }
          if let Some(ds) = DISCORDS.get(&guild_id.0.get()) {
            if let Some(log) = ds.log {
              let timestamp: Timestamp = chrono::Utc::now().to_rfc3339().parse().expect("wrong timestamp");
              if let Err(why) = log.send_message(&ctx, |m| m
                .embed(|e| {
                  e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                    .title(reason)
                    .description("ely used bad word again,\nignoring")
                    .timestamp(timestamp)
              })).await {
                error!("Failed to log ely {}, {why}", msg.author.name);
              }
            }
          }
          return;
        }
      }
    }
  }
  if let Some(ds) = DISCORDS.get(&guild_id.0.get()) {
    if let Some(log) = ds.log {
      let msg_link = if really_delete {
          format!("{}", msg.channel_id.mention())
        } else {
          msg.link()
        };
        let timestamp: Timestamp = chrono::Utc::now().to_rfc3339().parse().expect("wrong timestamp");
      if let Err(why) = log.send_message(&ctx, |m| m
        .embed(|e| {
          e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
           .title(reason)
           .description(&format!( "User UID: {}\n original message: {}\n{}"
                                , msg.author.id.0, &msg.content
                                , msg_link ))
           .timestamp(timestamp)
       })).await {
        error!("Failed to log on spam {}, {why}", msg.author.name);
      }
    }
  }
  if really_delete {
    if let Err(why) = &msg.delete(&ctx).await {
      error!("Error deleting spam {why}");
    }
    if let Err(why) =
      msg.author.direct_message(ctx, |m|
        m.content(&format!("your message was removed with reason: {reason}\n please contact moderators if you think it was done by mistake"))
      ).await {
        warn!("Error sending message to {} from spam blocker {why}", msg.author.name);
    }
  } else if let Err(why) =
    msg.author.direct_message(ctx, |m|
      m.content("please, try to avoid using bad words!")
    ).await {
    warn!("Error sending message to {} from spam blocker {why}", msg.author.name);
  }
}

fn scam_link_check(lowercase: &str) -> bool {
  let words = lowercase.split(LINK_SEPARATORS).collect::<Vec<&str>>();
  BAN_LIST.domains.iter()
                  .any(|c| words.iter().any(|w|
      !w.is_empty() && w.trim() == c
    )
  )
}

async fn scan_link( guild_id: &GuildId
                  , ctx: &Context
                  , msg: &Message
                  , lowercase: &str ) {
  if scam_link_check(lowercase) {
    delete( guild_id, ctx, msg, true, true
          , &format!("SCAM LINK BLOCKED {lowercase}") ).await;
  }
}

pub async fn spam_check(
      guild_id: &GuildId
    , ctx: &Context
    , msg: &Message) {
  let lowercase = msg.content.to_lowercase();
  scan_link(guild_id, ctx, msg, &lowercase).await;
  for embed in &msg.embeds {
    if let Some(url) = &embed.url {
      let lowercase_url = url.to_lowercase();
      scan_link(guild_id, ctx, msg, &lowercase_url).await;
    }
  }
  if SLUR.is_match(&lowercase) {
    delete( guild_id, ctx, msg, false, true
          , "SLUR USED" ).await;
  }
  if LIGHT_SLUR_LOL.is_match(&lowercase) {
    delete( guild_id, ctx, msg, false, false
          , "LIGHT SLUR USED" ).await;
  }
}

#[cfg(test)]
mod antispam_tests {
  use super::*;
  // TODO: this test ignored because cargo test is shit
  #[test]
  #[ignore]
  fn scam_links_test() {
    assert_eq!(
      scam_link_check("steamcommunity.com"), false
    );
    assert_eq!(
      scam_link_check("steamcommunity.co"), true
    );
    assert_eq!(
      scam_link_check("steamcommunity.com.ru"), true
    );
  }
  #[test]
  fn slurs_test() {
    assert_eq!(
      SLUR.is_match("retard"), false
    );
    assert_eq!(
      LIGHT_SLUR_LOL.is_match("bastard"), true
    );
    assert_eq!(
      LIGHT_SLUR_LOL.is_match("bitchute"), false
    );
    assert_eq!(
      SLUR.is_match("hi nigga"), true
    );
    assert_eq!(
      SLUR.is_match("Hello my friend"), false
    );
  }
}

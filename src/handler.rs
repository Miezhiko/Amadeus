use crate::{
  common::{
    msg::{ channel_message }
  },
  commands::voice,
  collections::overwatch::{ OVERWATCH, OVERWATCH_REPLIES }
};

use serenity::{
  model::{ event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message
         , id::GuildId, user::User },
  prelude::*,
};

use serenity::builder::CreateEmbed;

use rand::{
  Rng,
  thread_rng,
  seq::SliceRandom
};

use regex::Regex;

pub struct Handler;

impl EventHandler for Handler {
  fn ready(&self, ctx : Context, ready : Ready) {
    info!("Connected as {}", ready.user.name);
    voice::rejoin_voice_channel(&ctx);
  }
  fn resume(&self, _ctx : Context, _ : ResumedEvent) {
    info!("Resumed");
  }
  fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, /*mut*/ member: Member) {
    if let Ok(channels) = guild_id.channels(&ctx) {
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx) {
          name == "log"
        } else {
          false
        });
      if let Some((_, channel)) = log_channel {
        let user = member.user.read();
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            let mut e = e
              .author(|a| a.icon_url(&user.face()).name(&user.name))
              .title("has joined here, that message was too short so it's just random text");
            if let Some(ref joined_at) = member.joined_at {
              e = e.timestamp(joined_at);
            } e
        })) {
          error!("Failed to log new user {:?}", why);
        }
      }
    }
  }
  fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user : User, _ : Option<Member>) {
    if let Ok(channels) = guild_id.channels(&ctx) {
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx) {
          name == "log"
        } else {
          false
        });
      if let Some((_, channel)) = log_channel {
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&user.face()).name(&user.name))
              .title("has left however it's okay, I think it's fine")
              .timestamp(chrono::Utc::now().to_rfc3339())
            })) {
          error!("Failed to log leaving user {:?}", why);
        }
      }
    }
  }
  fn message(&self, ctx : Context, mut msg : Message) {
    if msg.is_own(&ctx) {
      if msg.content.to_lowercase() == "pong" {
        if let Err(why) = msg.edit(&ctx, |m| m.content("🅱enis!")) {
          error!("Failed to Benis {:?}", why);
        }
      }
      return;
    } else if msg.author.bot {
      /* Earlie Amadeus was not that agressive but now...
      let rnd = rand::thread_rng().gen_range(0, 2);
      if rnd == 1 || msg.content == "pong" {*/

      let is_file = msg.attachments.len() > 0;
      /* ok if that's  a file we just keep it 
      for file in &msg.attachments {
      } */

      if !is_file {
        if let Err(why) = &msg.delete(&ctx) {
          error!("Error replacing other bots {:?}", why);
        }
      } else {
        if let Ok(messages) = msg.channel_id.messages(&ctx, |r|
          r.limit(5)
        ) {
          for mmm in messages {
            if mmm.content.as_str().contains("Processing, this may take awhile") {
              if let Err(why) = mmm.delete(&ctx) {
                error!("Error removing processing message {:?}", why);
              }
            }
          }
        }
      }

      if !msg.content.is_empty() {
        channel_message(&ctx, &msg, &msg.content.as_str());
      }

      for embed in &msg.embeds {
        let mut not_stupid_zephyr = true;
        if (&embed.description).is_some() {
          if embed.description.as_ref().unwrap().contains("DiscordAPIError") {
            not_stupid_zephyr = false
          }
        }
        if not_stupid_zephyr {
          if let Err(why) = &msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              *e = CreateEmbed::from(embed.clone());
              e })
          }) {
            error!("Error replacing other bots embeds {:?}", why);
          }
        }
      }

    } else if let Some(find_char_in_words) = OVERWATCH.into_iter().find(|&c| {
        let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
        let is_overwatch = Regex::new(regex.as_str()).unwrap();
        is_overwatch.is_match(msg.content.as_str()) }) 
      {
        let mut rng = thread_rng();
        set! { ov_reply = OVERWATCH_REPLIES.choose(&mut rng).unwrap()
             , reply = format!("{} {}", ov_reply, find_char_in_words) };
        if let Err(why) = msg.channel_id.say(&ctx, reply) {
          error!("Error sending overwatch reply: {:?}", why);
        }
    } else {
      let regex_no_u = Regex::new(r"(^|\W)((?i)no u(?-i))($|\W)").unwrap();
      if regex_no_u.is_match(msg.content.as_str()) {
        let rnd = rand::thread_rng().gen_range(0, 2);
        if rnd == 1 {
          if let Err(why) = msg.channel_id.say(&ctx, "No u") {
            error!("Error sending no u reply: {:?}", why);
          }
        }
      }
    }
  }
}

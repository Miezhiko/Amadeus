use crate::{
  ai::chain,
  conf,
  common::{
    msg::{ channel_message }
  },
  commands::voice,
  collections::overwatch::{ OVERWATCH, OVERWATCH_REPLIES },
  collections::channels::AI_ALLOWED
};

use serenity::{
  model::{ event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message, gateway::Activity
         , id::GuildId, id::ChannelId, user::User },
  prelude::*,
  http::AttachmentType,
  builder::CreateEmbed
};

use std::borrow::Cow;

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
      let ai_text = chain::generate_english(&ctx, &guild_id, 500);
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx) {
          name == "log"
        } else {
          false
        });
      if let Some((_, channel)) = log_channel {
        let user = member.user.read();
        let title = format!("has joined here, {}", ai_text.as_str());
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            let mut e = e
              .author(|a| a.icon_url(&user.face()).name(&user.name))
              .title(title);
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
      let ai_text = chain::generate_english(&ctx, &guild_id, 500);
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx) {
          name == "log"
        } else {
          false
        });
      if let Some((_, channel)) = log_channel {
        let title = format!("has left, {}", ai_text.as_str());
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&user.face()).name(&user.name))
              .title(title)
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
      let mut is_file = false;
      for file in &msg.attachments {
        if let Ok(bytes) = file.download() {
          let cow = AttachmentType::Bytes {
            data: Cow::from(bytes),
            filename: String::from(&file.filename)
          };
          if let Err(why) = msg.channel_id.send_message(&ctx, |m| m.add_file(cow)) {
            error!("Failed to download and post attachment {:?}", why);
          } else {
            is_file = true;
          }
        }
      }
      if let Err(why) = &msg.delete(&ctx) {
        error!("Error replacing other bots {:?}", why);
      }
      if is_file {
        if let Ok(messages) = msg.channel_id.messages(&ctx, |r|
          r.limit(5)
        ) {
          for mmm in messages {
            if mmm.content.as_str().to_lowercase().contains("processing") {
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
    } else {
      if msg.guild(&ctx).is_some() {
        let is_admin =
          if let Some(member) = msg.member(&ctx.cache) {
            if let Ok(permissions) = member.permissions(&ctx.cache) {
              permissions.administrator()
            } else { false }
          } else {false };
        if !is_admin {
          let channel_id = msg.channel_id;
          let mut conf = conf::parse_config();
          let last_channel_conf =
            ChannelId( conf.last_channel_chat.parse::<u64>().unwrap_or(0) );
          if last_channel_conf != channel_id {
            conf.last_channel_chat = format!("{}", channel_id);
            conf::write_config(&conf);
          }
          // wakes up on any activity
          let rndx = rand::thread_rng().gen_range(0, 5);
          if rndx != 1 {
            ctx.set_activity(Activity::listening(&msg.author.name));
            ctx.online();
          } else {
            let activity = chain::generate(&ctx, &msg, 400);
            if !activity.is_empty() {
              ctx.set_activity(Activity::playing(&activity));
              ctx.idle();
            }
          }
          let channel_name =
            if let Some(ch) = msg.channel(&ctx) {
              ch.id().name(&ctx).unwrap_or(String::from(""))
            } else { String::from("") };
          if AI_ALLOWED.into_iter().any(|&c| c == channel_name.as_str()) {
            let rnd = rand::thread_rng().gen_range(0, 3);
            let mentioned_bot = (&msg.mentions).into_iter().any(|u| u.bot);
            if rnd == 1 && !mentioned_bot {
              chain::chat(&ctx, &msg, 5000);
            }
          }
        }
      }
      if let Some(find_char_in_words) = OVERWATCH.into_iter().find(|&c| {
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
}

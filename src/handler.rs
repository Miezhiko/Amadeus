use crate::{
  stains::gate,
  common::{
    types::AOptions,
    lang,
    msg::{ channel_message }
  },
  stains::ai::chain,
  collections::{
    base::REACTIONS,
    overwatch::{ OVERWATCH, OVERWATCH_REPLIES },
    channels::AI_ALLOWED
  },
  commands::voice
};

use serenity::{
  prelude::*,
  model::{
    id::{ EmojiId, GuildId },
    event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message, channel::ReactionType, gateway::Activity
         , user::User },
  http::AttachmentType,
  builder::CreateEmbed
};

use std::{
  borrow::Cow,
  sync::atomic::{ Ordering, AtomicBool }
};

use rand::{
  Rng,
  thread_rng,
  seq::SliceRandom
};

use regex::Regex;

pub static THREADS : AtomicBool = AtomicBool::new(false);

pub struct Handler {
  options : AOptions
}

impl Handler {
  pub fn new(opts: &AOptions) -> Handler {
    Handler {
      options: opts.clone()
    }
  }
}

impl EventHandler for Handler {
  fn ready(&self, ctx: Context, ready: Ready) {
    info!("Connected as {}", ready.user.name);
    voice::rejoin_voice_channel(&ctx, &self.options);

    let threads_check = THREADS.load(Ordering::Relaxed);
    if !threads_check {
      gate::behavior::activate(&ctx, &self.options);
      THREADS.store(true, Ordering::Relaxed);
    }
  }
  fn resume(&self, _ctx : Context, _ : ResumedEvent) {
    info!("Resumed");
  }
  fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, /*mut*/ member: Member) {
    if let Ok(channels) = guild_id.channels(&ctx) {
      let ai_text = chain::generate_with_language(&ctx, &guild_id, 666, false);
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx)
          { name == "log" } else { false });
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
      let ai_text = chain::generate_with_language(&ctx, &guild_id, 666, false);
      let log_channel = channels.iter().find(|&(c, _)|
        if let Some(name) = c.name(&ctx)
          { name == "log" } else { false });
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
      if !msg.content.is_empty() && !msg.content.starts_with("http") {
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
        let mentioned_bot = (&msg.mentions).into_iter().any(|u| u.bot);
        if !mentioned_bot {
          let is_admin =
            if let Some(member) = msg.member(&ctx.cache) {
              if let Ok(permissions) = member.permissions(&ctx.cache) {
                permissions.administrator()
              } else { false }
            } else {false };
          if !is_admin {
            gate::LAST_CHANNEL.store(msg.channel_id.as_u64().clone(), Ordering::Relaxed);
            // wakes up on any activity
            let rndx = rand::thread_rng().gen_range(0, 5);
            if rndx != 1 {
              ctx.set_activity(Activity::listening(&msg.author.name));
              ctx.online();
            } else {
              let activity = chain::generate(&ctx, &msg, 666);
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
              let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
              let rnd = rand::thread_rng().gen_range(0, activity_level);
              if rnd == 1 {
                chain::chat(&ctx, &msg, 7000);
              }
              let rnd2 = rand::thread_rng().gen_range(0, 2);
              if rnd2 == 1 {
                let mut rng = thread_rng();
                let (emoji_id, emji_name) = *REACTIONS.choose(&mut rng).unwrap();
                let reaction = ReactionType::Custom {
                  animated: false,
                  id: EmojiId(emoji_id),
                  name: Some(String::from(emji_name))
                };

                if let Some(ch) = msg.channel(&ctx) {
                  let g = ch.guild().unwrap();
                  let guild_id = g.read().guild_id;
                  if let Ok(guild) = guild_id.to_partial_guild(&ctx) {
                    if let Ok(mut member) = guild.member(&ctx, msg.author.id) {
                      if let Some(role) = guild.role_by_name("UNBLOCK AMADEUS") {

                        let normal_people_rnd = rand::thread_rng().gen_range(0, 9);
                        if normal_people_rnd == 1 || member.roles.contains(&role.id) {

                          if let Err(why) = msg.react(&ctx, reaction) {
                            error!("Failed to react: {:?}", why);
                            if why.to_string().contains("blocked") {
                              if !member.roles.contains(&role.id) {
                                if let Err(why) = member.add_role(&ctx, role) {
                                  error!("Failed to assign gay role {:?}", why);
                                } else {
                                  let repl = if lang::is_russian(&msg.content.as_str()) {
                                    format!("Ну чел {} явно меня не уважает", msg.author.name)
                                  } else {
                                    format!("Seems like {} doesn't respect me :(", msg.author.name)
                                  };
                                  channel_message(&ctx, &msg, repl.as_str());
                                  let new_nick = format!("Hater {}", msg.author.name);
                                  if let Err(why2) = guild.edit_member(&ctx, msg.author.id, |m| m.nickname(new_nick)) {
                                    error!("Failed to change user's nick {:?}", why2);
                                  }
                                }
                              }
                            }
                          } else {
                            if member.roles.contains(&role.id) {
                              if let Err(why) = member.remove_role(&ctx, role) {
                                error!("Failed to remove gay role {:?}", why);
                              } else {
                                let repl = if lang::is_russian(&msg.content.as_str()) {
                                  format!("Ну чел {} извини если что, давай останемся друзьями", msg.author.name)
                                } else {
                                  format!("Dear {} thank you for unblocking me, let be friends!", msg.author.name)
                                };
                                channel_message(&ctx, &msg, repl.as_str());
                                if let Err(why2) = guild.edit_member(&ctx, msg.author.id, |m| m.nickname("")) {
                                  error!("Failed to reset user's nick {:?}", why2);
                                }
                              }
                            }
                          }

                        }

                        if member.roles.contains(&role.id) {
                          let new_nick = format!("Hater {}", msg.author.name);
                          if let Err(why2) = guild.edit_member(&ctx, msg.author.id, |m| m.nickname(new_nick)) {
                            error!("Failed to change user's nick {:?}", why2);
                          }
                          if let Err(why) = &msg.delete(&ctx) {
                            error!("Error replacing bad people {:?}", why);
                          }
                          if !msg.content.is_empty() && !msg.content.starts_with("http") {
                            let new_words = chain::obfuscate(msg.content.as_str());
                            let says = if lang::is_russian(new_words.as_str()) {
                              "говорит"
                            } else { "says" };
                            let rm = format!("{} {} {} {}", msg.author.name, says, new_words, msg.content.as_str());
                            channel_message(&ctx, &msg, rm.as_str());
                          }
                        }

                        let rnd3 = rand::thread_rng().gen_range(0, 9);
                        if rnd3 != 1 {
                          if let Err(why) = msg.delete_reactions(&ctx) {
                            error!("Failed to remove all the reactions {:?}", why);
                          }
                        }
                      }
                    }
                  }
                }

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
}

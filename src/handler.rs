use crate::{
  types::{ common::CoreGuild
         , options::* },
  steins::{ gate
          , ai::{ cache, chain }
          , cyber::replay::{ replay_embed
                           , attach_replay }
          },
  common::{ db::trees
          , help::{ lang, channel::channel_by_name }
          , msg::channel_message
          , constants::{ LOG_CHANNEL
                       , UNBLOCK_ROLE
                       , LIVE_ROLE }
          },
  collections::{ base::{ REACTIONS, WHITELIST }
               , channels::{ AI_ALLOWED, EXCEPTIONS, IGNORED }
               },
  commands::music::rejoin_voice_channel
};

use serenity::{
  prelude::*,
  async_trait,
  utils::Colour,
  model::{ guild::ActionMessage
         , id::{ EmojiId, GuildId, MessageId, UserId, ChannelId }
         , event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message, channel::ReactionType, gateway::Activity
         , user::User
         },
  http::AttachmentType,
  builder::CreateEmbed
};

use std::{ borrow::Cow, collections::VecDeque
         , sync::atomic::{ Ordering, AtomicBool }
         , time::Duration
         };

use rand::{ Rng
          , seq::SliceRandom
          , rngs::StdRng
          , SeedableRng };

use regex::Regex;
use once_cell::sync::Lazy;

pub static THREADS: AtomicBool  = AtomicBool::new(false);
pub static BLAME: AtomicBool    = AtomicBool::new(false);
pub static RESTORE: AtomicBool  = AtomicBool::new(false);

pub struct Handler { ioptions:   IOptions
                   , roptions:   ROptions
                   , amadeus_id: UserId }

impl Handler {
  pub fn new(iopts: &IOptions, ropts: ROptions, amadeus: UserId) -> Handler {
    Handler { ioptions:   iopts.clone()
            , roptions:   ropts
            , amadeus_id: amadeus }
  }
}

pub static BACKUP: Lazy<Mutex<VecDeque<(MessageId, Message)>>> =
  Lazy::new(|| Mutex::new(VecDeque::with_capacity(64)));
pub static MUTED: Lazy<Mutex<Vec<UserId>>> =
  Lazy::new(|| Mutex::new(Vec::new()));

#[async_trait]
impl EventHandler for Handler {
  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    info!("Cache is READY");
    for guild_id in guilds {
      if guild_id.0 != self.ioptions.guild && guild_id.0 != self.ioptions.amadeus_guild {
        if let Some(serv) = self.ioptions.servers.iter().find(|s| s.id == guild_id.0) {
          if serv.kind == CoreGuild::Unsafe {
            if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
              if let Ok(member) = guild.member(&ctx, self.amadeus_id).await {
                if let Ok(some_permissions) = member.permissions(&ctx).await {
                  if some_permissions.administrator() {
                    if guild.role_by_name(UNBLOCK_ROLE).is_none() {
                      if let Err(why) =
                        // Hadouken
                        guild.create_role(&ctx,
                          |r| r.colour(Colour::from_rgb(226,37,37).0 as u64)
                              .hoist(false)
                              .mentionable(false)
                              .name(UNBLOCK_ROLE)).await {
                        error!("Failed to create UNBLOCK role, {:?}", why);
                      }
                    }
                    if guild.role_by_name(LIVE_ROLE).is_none() {
                      if let Err(why) =
                        guild.create_role(&ctx,
                          |r| r.colour(Colour::from_rgb(117,244,255).0 as u64)
                              .hoist(true)
                              .position(100) // bigger = higher
                              .mentionable(false)
                              .name(LIVE_ROLE)).await {
                        error!("Failed to create LIVE role, {:?}", why);
                      }
                    }
                  }
                }
              }
            }
          }
        } else {
          info!("leaving guild: {:?}", guild_id.0);
          if let Err(why) = guild_id.leave(&ctx).await {
            error!("Failed to leave guild {:?}", why);
          }
        }
      }
    }
    let threads_check = THREADS.load(Ordering::Relaxed);
    if !threads_check {
      THREADS.store(true, Ordering::Relaxed);
      gate::behavior::activate(ctx, &self.ioptions, &self.amadeus_id).await;
    }
  }
  async fn ready(&self, ctx: Context, ready: Ready) {
    info!("Connected as {}", ready.user.name);
    rejoin_voice_channel(&ctx, &self.roptions).await;
  }
  async fn resume(&self, _ctx: Context, _: ResumedEvent) {
    info!("Resumed");
  }
  async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {
    let mut muted_lock = MUTED.lock().await;
    if muted_lock.contains(&member.user.id) {
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, member.user.id).await {
          if let Some(role) = guild.role_by_name("muted") {
            if !member.roles.contains(&role.id) {
              if let Err(why) = member.add_role(&ctx, role).await {
                error!("Failed to assign muted role {:?}", why);
              } else {
                let mut found_users = vec![];
                for (i, u) in muted_lock.iter().enumerate() {
                  if *u == member.user.id {
                    found_users.push(i);
                  }
                }
                for i in found_users {
                  muted_lock.remove(i);
                }
              }
            }
          }
        }
      }
    }
  }
  async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _: Option<Member>) {
    let _was_on_chat = trees::clear_points(guild_id.0, user.id.0).await;
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Ok(member) = guild.member(&ctx, user.id).await {
        if let Some(role) = guild.role_by_name("muted") {
          if member.roles.contains(&role.id) {
            let mut muted_lock = MUTED.lock().await;
            if !muted_lock.contains(&member.user.id) {
              muted_lock.push(member.user.id);
            }
          }
        }
      }
    }
    if let Ok(channels) = guild_id.channels(&ctx).await {
      if let Some((channel, _)) = channel_by_name(&ctx, &channels, "main").await {
        let ai_text = chain::generate_with_language(&ctx, false).await;
        let title = format!("has left, {}", &ai_text);
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&user.face()).name(&user.name))
             .title(title)
             .timestamp(chrono::Utc::now().to_rfc3339())
            })).await {
          error!("Failed to log leaving user {:?}", why);
        }
      }
    }
  }
  async fn message_delete( &self
                         , ctx: Context
                         , channel_id: ChannelId
                         , deleted_message_id: MessageId
                         , _guild_id: Option<GuildId> ) {
    if RESTORE.load(Ordering::Relaxed) {
      if !AI_ALLOWED.iter().any(|c| c.id == channel_id.0) {
        return;
      }
      let backup_deq = BACKUP.lock().await;
      if !backup_deq.is_empty() {
        if let Some((_, msg)) = backup_deq.iter().find(|(id, _)| *id == deleted_message_id) {
          if msg.is_own(&ctx).await { // TODO: not sure whether we want to backup ALL
            if let Some(guild_id) = msg.guild_id {
              if let Ok(audit) = ctx.http.get_audit_logs( guild_id.0
                                                        , Some( ActionMessage::Delete as u8 )
                                                        , None
                                                        , None
                                                        , Some(1)).await {
                // Here we just hope it's last in Audit log, very unsafe stuff
                if let Some(entry) = audit.entries.values().next() {
                  // that entry contains target_id: Option<u64> but nobody knows what's that
                  if let Ok(deleter) = ctx.http.get_user(entry.user_id.0).await {
                    if !deleter.bot {
                      // message was removed by admin or by author
                      info!("{} or {} was trying to remove message", deleter.name, msg.author.name);
                      // log(&ctx, &guild_id, &log_text).await;
                      // But I don't allow it
                      for file in &msg.attachments {
                        if let Ok(bytes) = file.download().await {
                          let cow = AttachmentType::Bytes {
                            data: Cow::from(bytes),
                            filename: String::from(&file.filename)
                          };
                          if let Err(why) = channel_id.send_message(&ctx, |m| m.add_file(cow)).await {
                            error!("Failed to download and post attachment {:?}", why);
                          }
                        }
                      }
                      if !msg.content.is_empty() {
                        if let Err(why) = channel_id.send_message(&ctx, |m|
                            m.content(&msg.content)
                          ).await {
                          error!("Failed to post my message again, {:?}", why);
                        };
                      }
                      for embed in &msg.embeds {
                        if let Err(why) = channel_id.send_message(&ctx, |m| {
                          m.embed(|e| {
                            *e = CreateEmbed::from(embed.clone());
                            e })
                        }).await {
                          error!("Error reposting embed {:?}", why);
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        } else {
          warn!("Failed to get deleted message");
        }
      }
    }
  }
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.is_own(&ctx).await {
      let blame_check = BLAME.load(Ordering::Relaxed);
      if !blame_check {
        if let Some(g) = msg.guild_id {
          if self.ioptions.servers.iter().any(|s| s.id == g.0
                                               && s.kind == CoreGuild::Unsafe) {
            if let Ok(guild) = g.to_partial_guild(&ctx).await {
              if let Ok(member) = guild.member(&ctx, &msg.author.id).await {
                if let Ok(some_permissions) = member.permissions(&ctx).await {
                  if !some_permissions.administrator() {
                    BLAME.store(true, Ordering::Relaxed);
                    for _ in 0..10u8 {
                      channel_message(&ctx, &msg,
                        "Set administrator role for me, please!").await;
                    }
                    BLAME.store(false, Ordering::Relaxed);
                  }
                }
              }
            }
          }
        }
        if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0) {
          let mut backup_deq = BACKUP.lock().await;
          if backup_deq.len() == backup_deq.capacity() {
            backup_deq.pop_front();
          }
          backup_deq.push_back((msg.id, msg));
        }
      }
    } else if msg.author.bot {
      if let Some(g) = msg.guild_id {
        if self.ioptions.servers.iter()
                                .any(|s| s.id == g.0
                                      && s.kind == CoreGuild::Safe) {
          return;
        }
      }
      if EXCEPTIONS.contains(&msg.channel_id.0) {
        return;
      }
      let mut is_file = false;
      for file in &msg.attachments {
        if let Ok(bytes) = file.download().await {
          let cow = AttachmentType::Bytes {
            data: Cow::from(bytes),
            filename: String::from(&file.filename)
          };
          if let Err(why) = msg.channel_id.send_message(&ctx, |m| m.add_file(cow)).await {
            error!("Failed to download and post attachment {:?}", why);
          } else {
            is_file = true;
          }
        }
      }
      if let Err(why) = &msg.delete(&ctx).await {
        error!("Error replacing other bots {:?}", why);
      }
      if is_file {
        if let Ok(messages) = msg.channel_id.messages(&ctx, |r|
          r.limit(3)
        ).await {
          for mmm in messages {
            if mmm.content.to_lowercase().contains("processing") {
              if let Err(why) = mmm.delete(&ctx).await {
                error!("Error removing processing message {:?}", why);
              }
            }
          }
        }
      }
      if !msg.content.is_empty() && !msg.content.starts_with("http") {
        channel_message(&ctx, &msg, &msg.content).await;
      }
      for embed in &msg.embeds {
        let mut not_stupid_zephyr = true;
        
        if let Some(desc) = &embed.description {
          if desc.contains("DiscordAPIError") {
            not_stupid_zephyr = false;
          }
        }
        if not_stupid_zephyr {
          if let Err(why) = &msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| {
              *e = CreateEmbed::from(embed.clone());
              e })
          }).await {
            error!("Error replacing other bots embeds {:?}", why);
          }
        }
      }
    } else if !msg.content.starts_with('~') {
      if let Some(guild_id) = msg.guild_id {
        if IGNORED.contains(&msg.channel_id.0) {
          return;
        }
        if (&msg.mentions).iter().any(|u| u.bot) {
          if (&msg.mentions).iter().any(|u| u.bot && u.id == self.amadeus_id) {
            let amention1 = format!("<@{}>", self.amadeus_id);
            let amention2 = format!("<@!{}>", self.amadeus_id);
            if !msg.content.starts_with(&amention1)
            && !msg.content.starts_with(&amention2) {
              chain::response(&ctx, &msg).await;
            }
          }
        } else {
          trees::add_points(guild_id.0, msg.author.id.0, 1).await;
          for file in &msg.attachments {
            if file.filename.ends_with("w3g") {
              if msg.channel_id == LOG_CHANNEL {
                if !attach_replay(&ctx, &msg, file).await {
                  warn!("Failed to attach an replay to log!");
                } else {
                  info!("Relay attached successfully");
                }
                if let Err(why) = &msg.delete(&ctx).await {
                  error!("failed to clean attachment from log {:?}", why);
                }
              } else {
                let rainbow = ReactionType::Unicode(String::from("🌈"));
                let _ = msg.react(&ctx, rainbow).await;
                loop {
                  if let Some(reaction) =
                    &msg.await_reaction(&ctx)
                        .timeout(Duration::from_secs(3600)).await {
                    let emoji = &reaction.as_inner_ref().emoji;
                    if emoji.as_data().as_str() == "🌈" {
                      if let Err(why) = replay_embed(&ctx, &msg, file).await {
                        error!("Failed to analyze replay:\n{:?}", why);
                      }
                      if let Err(why) = msg.delete_reactions(&ctx).await {
                        error!("failed to delte msg reactions {:?}", why);
                      }
                    }
                  } else {
                    if let Err(why) = msg.delete_reactions(&ctx).await {
                      error!("failed to delte msg reactions {:?}", why);
                    }
                    break;
                  }
                }
              }
              return;
            }
          }
          gate::LAST_CHANNEL.store(msg.channel_id.0, Ordering::Relaxed);

          let rndx: u8 = rand::thread_rng().gen_range(0..3);
          if rndx != 1 {
            if let Some(nick) = msg.author.nick_in(&ctx, &guild_id).await {
              ctx.set_activity(Activity::listening(&nick)).await;
            } else {
              ctx.set_activity(Activity::listening(&msg.author.name)).await;
            }
            ctx.online().await;
          } else {
            let activity = chain::generate(&ctx, &msg, None).await;
            if !activity.is_empty() {
              if activity.contains('<') && activity.contains('>') {
                static RE_IB: Lazy<Regex> =
                  Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
                let replaced = RE_IB.replace_all(&activity, "");
                if !replaced.is_empty() {
                  ctx.set_activity(Activity::competing(&replaced)).await;
                }
              } else {
                ctx.set_activity(Activity::playing(&activity)).await;
              }
              ctx.idle().await;
            }
          }

          if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0) {
            let activity_level = cache::ACTIVITY_LEVEL.load(Ordering::Relaxed);
            let rnd = rand::thread_rng().gen_range(0..activity_level);
            if rnd == 1 {
              chain::chat(&ctx, &msg).await;
            }
            let rnd2: u16 = rand::thread_rng().gen_range(0..2);
            if rnd2 == 1 {
              let mut rng = StdRng::from_entropy();
              let emoji = REACTIONS.choose(&mut rng).unwrap();
              let reaction = ReactionType::Custom {
                animated: false,
                id: EmojiId(emoji.id),
                name: Some(emoji.name.clone())
              };

              if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
                if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
                  if let Some(role) = guild.role_by_name(UNBLOCK_ROLE) {

                    let normal_people_rnd: u16 = rand::thread_rng().gen_range(0..9);
                    if (normal_people_rnd == 1 || member.roles.contains(&role.id))
                    && !WHITELIST.iter().any(|u| *u == msg.author.id.0)
                    && !self.ioptions.servers.iter()
                                              .any(|s| s.id    == guild_id.0
                                                    && s.kind  == CoreGuild::Safe) {

                      if let Err(why) = msg.react(&ctx, reaction).await {
                        error!("Failed to react: {:?}", why);
                        if why.to_string().contains("blocked")
                        && !member.roles.contains(&role.id) {
                          if let Err(why) = member.add_role(&ctx, role).await {
                            error!("Failed to assign hater role {:?}", why);
                          } else {
                            let nick = member.nick.unwrap_or_else(|| msg.author.name.clone());
                            let repl = if lang::is_russian(&msg.content) {
                              format!("Ну чел {} явно меня не уважает", &nick)
                            } else {
                              format!("Seems like {} doesn't respect me :(", &nick)
                            };
                            channel_message(&ctx, &msg, &repl).await;
                            let new_nick: String = format!("Hater {}", &msg.author.name);
                            if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m|
                              m.nickname(new_nick)).await {
                              error!("Failed to change user's nick {:?}", why2);
                            }
                          }
                        }
                      } else if member.roles.contains(&role.id) {
                        if let Err(why) = member.remove_role(&ctx, role).await {
                          error!("Failed to remove gay role {:?}", why);
                        } else {
                          let repl = if lang::is_russian(&msg.content) {
                            format!("Пчел {} извини если что, давай останемся друзьями", msg.author.name)
                          } else {
                            format!("Dear {} thank you for unblocking me, let be friends!", msg.author.name)
                          };
                          channel_message(&ctx, &msg, &repl).await;
                          if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m| m.nickname("")).await {
                            error!("Failed to reset user's nick {:?}", why2);
                          }
                        }
                      }
                    }

                    if member.roles.contains(&role.id) {
                      let new_nick = format!("Hater {}", msg.author.name);
                      if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m| m.nickname(new_nick)).await {
                        error!("Failed to change user's nick {:?}", why2);
                      }
                      if let Err(why) = &msg.delete(&ctx).await {
                        error!("Error replacing bad people {:?}", why);
                      }
                      if !msg.content.is_empty() && !msg.content.starts_with("http") {
                        let new_words = chain::obfuscate(&msg.content);
                        let says = if lang::is_russian(&new_words) {
                          "говорит"
                        } else { "says" };
                        let rm = format!("{} {} {} {}", msg.author.name, says, new_words, &msg.content);
                        channel_message(&ctx, &msg, &rm).await;
                      }
                    }

                    let rnd3: u8 = rand::thread_rng().gen_range(0..9);
                    if rnd3 != 1 {
                      if let Err(why) = msg.delete_reactions(&ctx).await {
                        error!("Failed to remove all the reactions {:?}", why);
                      }
                    }
                  } else if let Err(why) =
                    guild.create_role(&ctx,
                        |r| r.colour(Colour::from_rgb(226,37,37).0 as u64)
                              .name(UNBLOCK_ROLE)).await {
                    error!("Failed to create UNBLOCK role, {:?}", why);
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      info!("User {} banned from {}", banned_user.name, guild.name);
    } else {
      info!("User {} banned from nowhere", banned_user.name);
    }
  }
  async fn guild_ban_removal(&self, ctx: Context, guild_id: GuildId, unbanned_user: User) {
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      info!("User {} unbanned from {}", unbanned_user.name, guild.name);
    } else {
      info!("User {} unbanned from nowhere", unbanned_user.name);
    }
  }
}

use crate::{
  message::{ self
           , RESTORE, BACKUP },
  types::{ common::CoreGuild
         , options::* },
  steins::{ gate
          , ai::chain
          },
  common::{ db::trees
          , help::channel::channel_by_name
          , constants::{ UNBLOCK_ROLE
                       , LIVE_ROLE }
          },
  collections::channels::AI_ALLOWED,
  commands::music::rejoin_voice_channel
};

use serenity::{
  prelude::*,
  async_trait,
  utils::Colour,
  model::{ guild::ActionMessage
         , id::{ GuildId, MessageId, UserId, ChannelId }
         , event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message
         , user::User
         },
  http::AttachmentType,
  builder::CreateEmbed
};

use std::{ borrow::Cow
         , sync::atomic::{ Ordering, AtomicBool }
         };

use once_cell::sync::Lazy;

pub static THREADS: AtomicBool  = AtomicBool::new(false);

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
    message::process( &self.ioptions
                    , self.amadeus_id
                    , &ctx
                    , msg ).await;
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

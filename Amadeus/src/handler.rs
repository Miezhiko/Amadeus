use crate::{
  slash::{ definitions, interactions },
  message::{ self
           , RESTORE, BACKUP },
  types::{ serenity::CoreGuild
         , options::* },
  common::{ options
          , db::trees::{ points, roles, emojis }
          , constants::{ UNBLOCK_ROLE
                       , LIVE_ROLE
                       , MUTED_ROLE, MUTED_ROOMS }
          },
  collections::{ team::DISCORDS
               , channels::AI_ALLOWED },
  commands::music::rejoin_voice_channel
};

#[cfg(feature = "trackers")]
use crate::steins::gate;

use serenity::{
  prelude::*,
  builder::*,
  async_trait,
  model::{ guild::audit_log::MessageAction
         , id::{ GuildId, MessageId, UserId, ChannelId, RoleId }
         , event::ResumedEvent, gateway::Ready, guild::Member
         , channel::{ Message, Reaction, ReactionType
                    , PermissionOverwrite, PermissionOverwriteType }
         , user::User, application::interaction::Interaction
         , permissions::Permissions
         , colour::Colour
         },
  builder::CreateAttachment
};

use std::{ sync::atomic::{ Ordering, AtomicBool }
         , collections::HashSet
         };

use once_cell::sync::Lazy;

pub static THREADS: AtomicBool  = AtomicBool::new(false);

pub struct Handler { ioptions:   IOptions
                   , roptions:   ROptions
                   , amadeus_id: UserId }

impl Handler {
  pub async fn new(iopts: IOptions, amadeus: UserId) -> anyhow::Result<Handler> {
    Ok(Handler { ioptions:   iopts
               , roptions:   options::get_roptions().await?
               , amadeus_id: amadeus })
  }
}

pub static MUTED: Lazy<Mutex<HashSet<UserId>>> =
  Lazy::new(|| Mutex::new(HashSet::new()));

#[async_trait]
impl EventHandler for Handler {

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    interactions::handle_slash_commands(&ctx, &interaction).await;
  }

  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    info!("Cache is READY");
    for guild_id in guilds {
      if guild_id.0.get() != self.ioptions.guild && guild_id.0.get() != self.ioptions.amadeus_guild {
        if let Some(serv) = self.ioptions.servers.iter().find(|s| s.id == guild_id.0.get()) {
          if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
            definitions::create_app_commands(&ctx, &guild).await;
            if let Ok(member) = guild.member(&ctx, self.amadeus_id).await {
              if let Ok(some_permissions) = member.permissions(&ctx) {
                if some_permissions.administrator() {
                  if serv.kind == CoreGuild::Unsafe {
                    if guild.role_by_name(UNBLOCK_ROLE).is_none() {
                      if let Err(why) =
                        guild.create_role(&ctx, EditRole::default()
                          .colour(Colour::from_rgb(226,37,37).0 as u32)
                          .hoist(false)
                          .mentionable(false)
                          .name(UNBLOCK_ROLE)).await {
                        error!("Failed to create UNBLOCK role, {why}");
                      }
                    }
                    if guild.role_by_name(LIVE_ROLE).is_none() {
                      if let Err(why) =
                        guild.create_role(&ctx, EditRole::default()
                          .colour(Colour::from_rgb(117,244,255).0 as u32)
                          .hoist(true)
                          .position(100) // bigger = higher
                          .mentionable(false)
                          .name(LIVE_ROLE)).await {
                        error!("Failed to create LIVE role, {why}");
                      }
                    }
                    if guild.role_by_name(MUTED_ROLE).is_none() {
                      if let Err(why) =
                        guild.create_role(&ctx, EditRole::default()
                          .colour(Colour::from_rgb(113,113,113).0 as u32)
                          .hoist(true)
                          .position(100) // bigger = higher
                          .mentionable(false)
                          .name(MUTED_ROLE)).await {
                        error!("Failed to create muted role, {why}");
                      }
                    }
                  }
                  if let Some(muted_role) = guild.role_by_name(MUTED_ROLE) {
                    if let Ok(channels) = guild.channels(&ctx).await {
                      let deny = Permissions::SEND_MESSAGES
                                | Permissions::ADD_REACTIONS
                                | Permissions::STREAM
                                | Permissions::SEND_TTS_MESSAGES
                                | Permissions::ATTACH_FILES
                                | Permissions::EMBED_LINKS
                                | Permissions::SPEAK
                                | Permissions::CHANGE_NICKNAME
                                | Permissions::MANAGE_EMOJIS_AND_STICKERS
                                | Permissions::USE_APPLICATION_COMMANDS
                                | Permissions::CREATE_PUBLIC_THREADS
                                | Permissions::CREATE_PRIVATE_THREADS
                                | Permissions::USE_VAD
                                | Permissions::SEND_MESSAGES_IN_THREADS;
                      let overwrite = PermissionOverwrite {
                        allow: Permissions::empty(), deny,
                        kind: PermissionOverwriteType::Role(muted_role.id)
                      };
                      for (chan, _guild_channel) in channels {
                        if MUTED_ROOMS.contains(&chan) {
                          continue;
                        }
                        if let Err(why) = chan.create_permission(&ctx, overwrite.clone()).await {
                          error!("Failed to create channel override for muted role, {why}");
                        }
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
            error!("Failed to leave guild {why}");
          }
        }
      } else {
        // this is for own server and amadeus testing server
        if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
          definitions::create_app_commands(&ctx, &guild).await;
        }
      }
    }
    let threads_check = THREADS.load(Ordering::Relaxed);
    if !threads_check {
      THREADS.store(true, Ordering::Relaxed);
      #[cfg(feature = "trackers")]
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

  async fn guild_member_addition(&self, ctx: Context, mut member: Member) {
    let mut muted_lock = MUTED.lock().await;
    if muted_lock.contains(&member.user.id) {
      if let Ok(guild) = member.guild_id.to_partial_guild(&ctx).await {
        if let Some(role) = guild.role_by_name(MUTED_ROLE) {
          if !member.roles.contains(&role.id) {
            if let Err(why) = member.add_role(&ctx, role).await {
              error!("Failed to assign muted role {why}");
            } else {
              muted_lock.remove(&member.user.id);
            }
          }
        }
      }
    }
    if let Ok(roles) = roles::restore_roles( &member.guild_id.get()
                                           , &member.user.id.get() ).await {
      for role in roles {
        if let Err(why) = member.add_role(&ctx, role).await {
          error!("Failed to reset role {} for user {} on {}, {why}", role, member.user.name, member.guild_id);
        }
      }
    }
  }

  async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, m: Option<Member>) {
    if let Err(why) = points::clear_points(guild_id.0.get(), user.id.0.get()).await {
      error!("some problem with points: {why}");
    }
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Some(member) = m {
        if let Some(role) = guild.role_by_name(MUTED_ROLE) {
          if member.roles.contains(&role.id) {
            let mut muted_lock = MUTED.lock().await;
            if !muted_lock.contains(&member.user.id) {
              muted_lock.insert(member.user.id);
            }
          }
        }
      }
    }
    if let Some(ds) = DISCORDS.get(&guild_id.0.get()) {
      if let Some(log) = ds.log {
        if let Err(why) = log.send_message(&ctx, CreateMessage::new()
          .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(&user.name).icon_url(&user.face()))
            .title(&format!("has left (or was kicked)\nUID: {}", user.id.0))
            .timestamp(chrono::Utc::now())
          )).await {
          error!("Failed to log leaving user {why}");
        }
      }
    }
  }

  async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
    if let ReactionType::Custom{animated: _, id, name: _} = &add_reaction.emoji {
      if let Some(user_id) = &add_reaction.user_id {
        if let Ok(msg) = &add_reaction.message(&ctx).await {
          if let Ok(channel) = msg.channel(&ctx).await {
            if let Some(guild_channel) = channel.guild() {
              let user_u64 = user_id.get();
              let guild_u64 = guild_channel.guild_id.get();
              if let Ok(Some(emoji_roles)) = emojis::message_roles(&guild_u64, &add_reaction.message_id.get()).await {
                if let Some(role) = emoji_roles.get(&id.get()) {
                  if let Ok(guild) = guild_channel.guild_id.to_partial_guild(&ctx).await {
                    if let Ok(mut member) = guild.member(&ctx, user_id).await {
                      let role_id = RoleId( to_nzu!( *role ) );
                      if !member.roles.contains(&role_id) {
                        if let Err(why) = member.add_role(&ctx, role_id).await {
                          error!("Failed to assign role {why}");
                        } else {
                          let mut roles_vector : Vec<u64> = Vec::new();
                          for role in &member.roles {
                            roles_vector.push(role.get());
                          }
                          roles::update_roles(&guild_u64, &user_u64, &roles_vector).await;
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
    }
  }

  async fn reaction_remove(&self, ctx: Context, add_reaction: Reaction) {
    if let ReactionType::Custom{animated: _, id, name: _} = &add_reaction.emoji {
      if let Some(user_id) = &add_reaction.user_id {
        if let Ok(msg) = &add_reaction.message(&ctx).await {
          if let Ok(channel) = msg.channel(&ctx).await {
            if let Some(guild_channel) = channel.guild() {
              let user_u64 = user_id.get();
              let guild_u64 = guild_channel.guild_id.get();
              if let Ok(Some(emoji_roles)) = emojis::message_roles(&guild_u64, &add_reaction.message_id.get()).await {
                if let Some(role) = emoji_roles.get(&id.get()) {
                  if let Ok(guild) = guild_channel.guild_id.to_partial_guild(&ctx).await {
                    if let Ok(mut member) = guild.member(&ctx, user_id).await {
                      let role_id = RoleId( to_nzu!( *role ) );
                      if member.roles.contains(&role_id) {
                        if let Err(why) = member.remove_role(&ctx, role_id).await {
                          error!("Failed to remove role {why}");
                        } else {
                          let mut roles_vector : Vec<u64> = Vec::new();
                          for role in &member.roles {
                            roles_vector.push(role.get());
                          }
                          roles::update_roles(&guild_u64, &user_u64, &roles_vector).await;
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
    }
  }

  async fn message_delete( &self
                         , ctx: Context
                         , channel_id: ChannelId
                         , deleted_message_id: MessageId
                         , _guild_id: Option<GuildId> ) {

    if RESTORE.load(Ordering::Relaxed) {
      if !AI_ALLOWED.iter().any(|c| c.id == channel_id.0.get()) {
        return;
      }
      let backup_deq = BACKUP.lock().await;
      if !backup_deq.is_empty() {
        if let Some((_, msg)) = backup_deq.iter().find(|(id, _)| *id == deleted_message_id) {
          if msg.is_own(&ctx) { // TODO: not sure whether we want to backup ALL
            if let Some(guild_id) = msg.guild_id {
              if let Ok(audit) = ctx.http.get_audit_logs( guild_id
                                                        , Some( MessageAction::Delete as u8 )
                                                        , None
                                                        , None
                                                        , Some(1)).await {
                // Here we just hope it's last in Audit log, very unsafe stuff
                for entry in audit.entries {
                  // that entry contains target_id: Option<u64> but nobody knows what's that
                  if let Ok(deleter) = ctx.http.get_user(entry.user_id).await {
                    if !deleter.bot {
                      // message was removed by admin or by author
                      info!("{} or {} was trying to remove message", deleter.name
                                                                   , msg.author.name);
                      // But I don't allow it
                      for file in &msg.attachments {
                        if let Ok(bytes) = file.download().await {
                          let cow = CreateAttachment::bytes(
                            &bytes,
                            String::from(&file.filename)
                          );
                          if let Err(why) = channel_id.send_message(&ctx, CreateMessage::new().add_file(cow)).await {
                            error!("Failed to download and post attachment {why}");
                          }
                        }
                      }
                      if !msg.content.is_empty() {
                        if let Err(why) = channel_id.send_message(&ctx, CreateMessage::new()
                            .content(&msg.content)
                          ).await {
                          error!("Failed to post my message again, {why}");
                        };
                      }
                      for embed in &msg.embeds {
                        if let Err(why) = channel_id.send_message(&ctx, CreateMessage::new()
                          .embed( CreateEmbed::from(embed.clone() ) )
                        ).await {
                          error!("Error reposting embed {why}");
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

  async fn message( &self
                  , ctx: Context
                  , msg: Message ) {
    message::process( &self.ioptions
                    , self.amadeus_id
                    , &ctx
                    , msg ).await;
  }

  async fn guild_ban_addition( &self, ctx: Context
                             , guild_id: GuildId
                             , banned_user: User ) {
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      info!("User {} banned from {}", banned_user.name, guild.name);
    } else {
      info!("User {} banned from nowhere", banned_user.name);
    }
  }

  async fn guild_ban_removal( &self, ctx: Context
                            , guild_id: GuildId
                            , unbanned_user: User ) {
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      info!("User {} unbanned from {}", unbanned_user.name, guild.name);
    } else {
      info!("User {} unbanned from nowhere", unbanned_user.name);
    }
  }

}

use crate::{
  types::serenity::IContext,
  common::{
    db::trees::{ emojis, roles },
    msg::{ channel_message
         , direct_message },
    system
  },
  steins::gate
};

#[cfg(not(target_os = "windows"))]
use crate::{
  steins::ai::cache::{ ACTIVITY_LEVEL
                     , actualize_cache
                     , clear_cache }
};

use serenity::{
  prelude::*,
  builder::GetMessages,
  model::channel::{ Message
                  , ReactionType },
  model::id::{ ChannelId
             , RoleId },
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use std::sync::{
  atomic::Ordering,
  Arc
};

use tokio::process::Command;

#[command]
#[min_args(2)]
#[owners_only]
async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  if let Ok(property) = args.single::<String>() {
    #[allow(clippy::single_match)]
    match property.as_str() {
      #[cfg(not(target_os = "windows"))]
      "activity" =>
        if let Ok(level) = args.single::<u32>() {
          ACTIVITY_LEVEL.store(level, Ordering::Relaxed);
          let chan_msg = format!("Activity level is: {level} now");
          channel_message(ctx, msg, &chan_msg).await;
        },
      "lsm" =>
        if let Ok(on_off) = args.single::<String>() {
          if let Some(on_off_bool) = match on_off.to_lowercase().as_str() {
                                      "on"  => Some(true),
                                      "off" => Some(false),
                                      _     => None
                                    } {
            let mut must_rewrite = false;
            {
              let data = ctx.data.read().await;
              if let Some(icontext) = data.get::<IContext>() {
                if icontext.lazy_static_models != on_off_bool {
                  must_rewrite = true;
                }
              }
            }
            if must_rewrite {
              let mut data = ctx.data.write().await;
              if let Some(icontext) = data.get_mut::<IContext>() {
                *icontext = Arc::new( IContext { lazy_static_models: on_off_bool } );
              }
            }
          }
        },
      _ => ()
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[owners_only]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  let last_channel_u64 = gate::LAST_CHANNEL.load(Ordering::Relaxed);
  if last_channel_u64 != 0 {
    let last_channel_conf = ChannelId( to_nzu!(last_channel_u64) );
    if msg.guild_id.is_some() {
      let text = args.message();
      if !text.is_empty() {
        if let Err(why) = last_channel_conf.say(ctx, text).await {
          error!("Failed say {why}");
        }
      }
    }
  }
  Ok(())
}

#[command]
#[owners_only]
async fn clear_messages(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if args.len() == 1 {
    let countdown: u8 = args.find().unwrap_or_default();
    if let Ok(vec) = msg.channel_id.messages(ctx, GetMessages::default().before(msg.id).limit(countdown)).await {
      let mut vec_id = Vec::new();
      for message in vec {
        vec_id.push(message.id);
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, msg, &format!("Deleted {countdown} messages")).await;
  } else if args.len() == 2 {
    let countdown: usize = args.find().unwrap_or_default();
    let counter: usize = args.find().unwrap_or_default();
    let full = countdown + counter;
    if let Ok(vec) = msg.channel_id.messages(ctx, GetMessages::default().before(msg.id).limit(full as u8)).await {
      let mut vec_id = Vec::new();
      for (i, message) in vec.iter().rev().enumerate() {
        if i < countdown {
          vec_id.push(message.id);
        }
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, msg, &format!("Deleted {countdown} messages")).await;
  }
  Ok(())
}

#[command]
#[owners_only]
async fn clear_chain_cache(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  #[cfg(not(target_os = "windows"))]
  clear_cache().await;
  channel_message(ctx, msg, "Cache removed").await;
  Ok(())
}

#[command]
#[owners_only]
async fn update_cache(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  #[cfg(not(target_os = "windows"))]
  actualize_cache(ctx, true).await;
  channel_message(ctx, msg, "Cache updated").await;
  Ok(())
}

#[command]
#[owners_only]
async fn upgrade(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  if let Err(why) = system::upgrade::upgrade_amadeus(ctx, &msg.channel_id).await {
    error!("Failed to upgrade Amadeus, {why}");
  }
  Ok(())
}

#[command]
#[owners_only]
async fn twitch_token_update(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  if system::hacks::twitch_update(ctx).await.is_ok() {
    channel_message(ctx, msg, "twitch access token updated").await;
  }
  Ok(())
}

// ~register_role 913474954111094844 913443636245561344 913469701529698334
#[command]
#[owners_only]
#[min_args(3)]
async fn register_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  if let Some(guild_id) = msg.guild_id {
    let message_id = args.single::<u64>()?;
    let emoji_id = args.single::<u64>()?;
    let role_id = args.single::<u64>()?;
    emojis::register_message( &guild_id.get()
                            , &message_id
                            , &emoji_id
                            , &role_id ).await;
    direct_message(ctx, msg, &format!("Message role {role_id} registered")).await;
  }
  Ok(())
}

#[command]
#[owners_only]
#[min_args(1)]
async fn list_message_roles(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command, {why}");
  }
  if let Some(guild_id) = msg.guild_id {
    let message_id = args.single::<u64>()?;
    if let Ok(Some(mr)) = emojis::message_roles( &guild_id.get()
                                               , &message_id ).await {
      channel_message(ctx, msg, &format!("message roles: {:?}", mr)).await;
    }
  }
  Ok(())
}

#[command]
#[owners_only]
async fn unban_all(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    let guild = guild_id.to_partial_guild(ctx).await?;
    let bans = guild.bans(ctx).await?;
    for ban in bans {
      if let Err(why) = guild.unban(ctx, ban.user.id).await {
        error!("Failed to unban user: {}, {why}", ban.user.name);
      }
    }
    channel_message(ctx, msg, "Everyone unbanned =_=").await;
  }
  Ok(())
}

#[command]
#[owners_only]
#[min_args(1)]
async fn eix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Ok(package) = args.single::<String>() {
    let eix_command = format!("eix \"{package}\"");
    let eix = Command::new("sh")
      .arg("-c")
      .arg(&eix_command)
      .output()
      .await
      .expect("failed to run eix");
    if let Ok(eix_out) = &String::from_utf8(eix.stdout) {
      if !eix_out.is_empty() {
        msg.reply(ctx, &format!("```{}```", &eix_out)).await?;
      }
    }
  }
  Ok(())
}

#[command]
#[owners_only]
#[min_args(2)]
async fn catch_up_with_roles(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
  set!{ chan_id = args.single::<u64>()?
      , msg_id  = args.single::<u64>()?
      , msg     = ctx.http.get_message(chan_id, msg_id).await?
      , channel = msg.channel(&ctx).await? };
  if let Some(guild_channel) = channel.guild() {
    let guild_u64 = guild_channel.guild_id.get();
    let guild = guild_channel.guild_id.to_partial_guild(ctx).await?;
    if let Ok(Some(emoji_roles)) = emojis::message_roles(&guild_u64, &msg_id).await {
      let mut reaction_types = vec![];
      for reaction in &msg.reactions {
        if let ReactionType::Custom{animated: _, id, name: _} = &reaction.reaction_type {
          if let Some(role) = emoji_roles.get(&id.get()) {
            reaction_types.push( (reaction.reaction_type.clone(), *role) );
          }
        }
      }
      for (rt, role) in reaction_types {
        if let Ok(rt_users) = msg.reaction_users(ctx, rt, None, None).await {
          for user in rt_users {
            let user_u64 = user.id.get();
            if let Ok(mut member) = guild.member(&ctx, user.id).await {
              let role_id = RoleId(to_nzu!(role));
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
  Ok(())
}

#[command]
#[owners_only]
#[min_args(1)]
async fn ban(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to ban").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      let guild = guild_id.to_partial_guild(&ctx).await?;
      if let Ok(member) = guild.member(&ctx, target_user.id).await {
        member.ban_with_reason(ctx, 0, &format!("banned by {}", msg.author.name)).await?;
      }
    }
  }
  Ok(())
}

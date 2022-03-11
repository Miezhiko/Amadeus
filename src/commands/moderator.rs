use crate::{
  collections::team::DISCORDS,
  common::{
    constants::MUTED_ROLE,
    msg::channel_message
  }
};

use serenity::{
  model::{ id::{ ChannelId, UserId, RoleId }
         , channel::*
         , permissions::Permissions },
  prelude::*,
  framework::standard::{ CommandResult
                       , macros::command
                       , Args }
};

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to mute").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      let guild = guild_id.to_partial_guild(&ctx).await?;
      let mut member = guild.member(&ctx, target_user.id).await?;
      if let Some(role) = guild.role_by_name(MUTED_ROLE) {
        if !member.roles.contains(&role.id) {
          if let Err(why) = member.add_role(&ctx, role).await {
            error!("Failed to assign muted role {why}");
          }
        }
      }
    }
  }
  Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to unmute").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      let guild = guild_id.to_partial_guild(&ctx).await?;
      let mut member = guild.member(&ctx, target_user.id).await?;
      if let Some(role) = guild.role_by_name(MUTED_ROLE) {
        if member.roles.contains(&role.id) {
          if let Err(why) = member.remove_role(&ctx, role).await {
            error!("Failed to unassign muted role {why}");
          }
        }
      }
    }
  }
  Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn move_discussion(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ target_channel = ChannelId( args.single::<u64>()? )
      , channel = target_channel.to_channel(ctx).await?
      , msg_guild_id = msg.guild_id.unwrap_or_default() };
  match channel.guild() {
    Some(guild_channel) => {
      if guild_channel.guild_id != msg_guild_id {
        return Err("Can't move discussion across servers".into());
      }
    },
    None => {
      return Err("Only work for guild channels".into());
    }
  };

  let comefrom_message = format!(
    "**Discussion moved here from {}**\n{}",
    msg.channel_id.mention(),
    msg.link_ensured(ctx).await
  );

  let comefrom_message = target_channel
    .send_message(ctx, |f| {
        f.content(comefrom_message)
    }).await?;

  channel_message(ctx, msg, &format!(
    "**{} suggested to move this discussion to {}**\n{}",
    &msg.author.name,
    target_channel.mention(),
    comefrom_message.link_ensured(ctx).await
  )).await;

  Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn timeout_to(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ user_id = UserId( args.single::<u64>()? )
      , msg_guild_id = msg.guild_id.unwrap_or_default() };

  let reason = if args.len() > 1 {
    if let Ok(r) = args.single::<String>() {
      Some(r)
    } else { None }
  } else { None };

  let guild = msg_guild_id.to_partial_guild(ctx).await?;
  if let Ok(mut member) = guild.member(ctx, user_id).await {
    let timeout = chrono::Utc::now() + chrono::Duration::hours(1);
    member.disable_communication_until_datetime(ctx, timeout).await?;
    let allow = Permissions::SEND_MESSAGES | Permissions::READ_MESSAGES;
    let deny = Permissions::READ_MESSAGES;
    let overwrite_user = PermissionOverwrite {
      allow, deny: Permissions::empty(),
      kind: PermissionOverwriteType::Member(member.user.id)
    };
    let overwrite_all = PermissionOverwrite {
      allow: Permissions::empty(), deny,
      kind: PermissionOverwriteType::Role( RoleId( msg_guild_id.0 ))
    };
    let mut permisssions_vec = vec![overwrite_user, overwrite_all];
    if let Some(muted_role) = guild.role_by_name(MUTED_ROLE) {
      let allow_muted = Permissions::SEND_MESSAGES;
      let muted_override = PermissionOverwrite {
        allow: allow_muted, deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role( muted_role.id )
      };
      permisssions_vec.push(muted_override);
    }
    /*if let Some(mod_role) = guild.role_by_name("mod") {
      let allow_mod = Permissions::MANAGE_CHANNELS;
      let mod_override = PermissionOverwrite {
        allow: allow_mod, deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role( mod_role.id )
      };
      permisssions_vec.push(mod_override);
    }*/
    let timeout_channel =
      guild.create_channel(ctx, |c| c.name(&format!("{}_timeout", member.user.name))
                                      .permissions(permisssions_vec)
                                      .kind(ChannelType::Text)).await?;

    timeout_channel.send_message(&ctx, |m| m
      .embed(|e| {
        let mut e =
          e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
            .title(&format!("{} you was timed out by {}", member.user.name, msg.author.name))
            .timestamp(chrono::Utc::now().to_rfc3339());
        if let Some(r) = &reason {
          e = e.description(r);
        } e
      })).await?;
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0) {
      if let Some(log) = ds.log {
        log.send_message(ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
              .title(&format!("{} timed out {}", msg.author.name, member.user.name))
              .timestamp(chrono::Utc::now().to_rfc3339())
          })).await?;
      }
    }
  } else {
    return Err("User is not member of guild".into());
  }

  Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn untimeout(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ user_id = UserId( args.single::<u64>()? )
      , msg_guild_id = msg.guild_id.unwrap_or_default() };

  let guild = msg_guild_id.to_partial_guild(ctx).await?;
  if let Ok(mut member) = guild.member(ctx, user_id).await {
    member.enable_communication(ctx).await?;
    msg.channel_id.send_message(&ctx, |m| m
      .embed(|e| {
        e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
          .title(&format!("{} was untimeouted out by {}", member.user.name, msg.author.name))
          .timestamp(chrono::Utc::now().to_rfc3339())
        })).await?;
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0) {
      if let Some(log) = ds.log {
        log.send_message(ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
              .title(&format!("{} removed time out from {}", msg.author.name, member.user.name))
              .timestamp(chrono::Utc::now().to_rfc3339())
            })).await?;
      }
    }
  } else {
    return Err("User is not member of guild".into());
  }

  Ok(())
}

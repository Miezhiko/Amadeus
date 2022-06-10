use crate::{
  collections::team::DISCORDS,
  common::{
    constants::MUTED_ROLE,
    msg::channel_message,
    help::channel::channel_by_name
  }
};

use serenity::{
  model::{ id::{ ChannelId, UserId, RoleId }
         , channel::*, Timestamp
         , permissions::Permissions },
  prelude::*,
  framework::standard::{ CommandResult
                       , macros::command
                       , Args }
};

const PURGE_ITERATIONS: usize = 10;

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
async fn timeout(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ user_id = UserId( args.single::<u64>()? )
      , msg_guild_id = msg.guild_id.unwrap_or_default() };

  let reason = if args.len() > 1 {
    Some( args.rest() )
  } else { None };

  let guild = msg_guild_id.to_partial_guild(ctx).await?;
  if let Ok(mut member) = guild.member(ctx, user_id).await {
    let allow = Permissions::SEND_MESSAGES | Permissions::VIEW_CHANNEL;
    let deny = Permissions::VIEW_CHANNEL;
    let overwrite_user = PermissionOverwrite {
      allow, deny: Permissions::empty(),
      kind: PermissionOverwriteType::Member(member.user.id)
    };
    let overwrite_moderator = PermissionOverwrite {
      allow, deny: Permissions::empty(),
      kind: PermissionOverwriteType::Member(msg.author.id)
    };
    let overwrite_all = PermissionOverwrite {
      allow: Permissions::empty(), deny,
      kind: PermissionOverwriteType::Role( RoleId( msg_guild_id.0 ))
    };
    let mut permisssions_vec = vec![overwrite_user, overwrite_moderator, overwrite_all];
    if let Some(muted_role) = guild.role_by_name(MUTED_ROLE) {
      member.add_role(ctx, muted_role).await?;
      let allow_muted = Permissions::SEND_MESSAGES;
      let muted_override = PermissionOverwrite {
        allow: allow_muted, deny: Permissions::empty(),
        kind: PermissionOverwriteType::Role( muted_role.id )
      };
      permisssions_vec.push(muted_override);
    } else {
      let timeout = chrono::Utc::now() + chrono::Duration::hours(1);
      member.disable_communication_until_datetime(ctx, Timestamp::from( timeout )).await?;
    }
    let channel_name = format!("{}_timeout", member.user.name);
    let timeout_channel =
      guild.create_channel(ctx, |c| c.name(&channel_name)
                                     .permissions(permisssions_vec)
                                     .rate_limit_per_user(2*60) // seconds
                                     .kind(ChannelType::Text)).await?;

    timeout_channel.send_message(&ctx, |m| m
      .content(&format!("{}\n", member.mention()))
      .embed(|e| {
        let mut e =
          e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
           .title(&format!("You was timed out by {}", msg.author.name))
           .timestamp(chrono::Utc::now().to_rfc3339());
        if let Some(r) = reason {
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
             .footer(|f| f.text(&format!("~j {}", &timeout_channel.name)))
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
async fn j(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ channel_name = args.single::<String>()?
      , msg_guild_id = msg.guild_id.unwrap_or_default() };
  if let Ok(channels) = msg_guild_id.channels(ctx).await {
    if let Some((channel, _)) = channel_by_name(ctx, &channels, &channel_name).await {
      let allow = Permissions::SEND_MESSAGES | Permissions::VIEW_CHANNEL;
      let overwrite_moderator = PermissionOverwrite {
        allow, deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(msg.author.id)
      };
      channel.create_permission(ctx, &overwrite_moderator).await?;
      if let Err(why) = channel.send_message(ctx, |m| m.content(&format!("{} has joined", msg.author.name))).await {
        error!("Failed to log new user {why}");
      }
      if let Some(ds) = DISCORDS.get(&msg_guild_id.0) {
        if let Some(log) = ds.log {
          log.send_message(ctx, |m| m
            .embed(|e| {
              e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
               .title(&format!("{} joined {}", msg.author.name, &channel_name))
               .timestamp(chrono::Utc::now().to_rfc3339())
            })).await?;
        }
      }
    }
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
    if let Some(muted_role) = guild.role_by_name(MUTED_ROLE) {
      if member.roles.contains(&muted_role.id) {
        member.remove_role(ctx, muted_role).await?;
      }
    }
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

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn prison(ctx: &Context, msg: &Message) -> CommandResult {
  set!{ msg_guild_id = msg.guild_id.unwrap_or_default()
      , guild = msg_guild_id.to_partial_guild(ctx).await?
      , channels = guild.channels(&ctx).await? };
  let mut prison_channels = vec![];
  for (chan_id, chan) in channels {
    if chan.name.ends_with("_timeout") {
      let name = chan.name.replace("_timeout", "");
      prison_channels.push(format!("{name}: {}", chan_id.mention()));
    }
  }
  if !prison_channels.is_empty() {
    channel_message(ctx, msg, prison_channels.join("\n").as_str()).await;
  } else {
    channel_message(ctx, msg, "no prison rooms found").await;
  }
  Ok(())
}

#[command]
#[required_permissions(BAN_MEMBERS)]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let mut users = vec![];
  for arg in args.iter::<u64>() {
    if let Ok(id) = &arg {
      users.push( *id );
    }
  }
  if users.is_empty() {
    channel_message(ctx, msg, "no users found").await;
    return Ok(());
  }
  let mut last_msg_id_on_iteration = Some(msg.id);
  let mut messages = std::collections::HashSet::new();
  for _iteration in [0..PURGE_ITERATIONS] {
    if let Some(last_msg_id) = last_msg_id_on_iteration {
      if let Ok(msgs) = msg.channel_id.messages(ctx,
          |g| g.before(last_msg_id).limit(255u8)
        ).await {
        for message in &msgs {
          if users.iter().any(|u| *u == message.author.id.0) {
            messages.insert(message.id);
          }
        }
        last_msg_id_on_iteration =
          // not fully sure about messages order :|
          if let Some(last_msg) = msgs.last() {
            Some( last_msg.id )
          } else {
            None
          };
      }
    }
  }

  if !messages.is_empty() {
    let messages_vec = Vec::from_iter(messages);
    msg.channel_id.delete_messages(ctx, messages_vec.as_slice()).await?;
    if let Err(why) = msg.delete(ctx).await {
      error!("Error deleting original command, {why}");
    }
    let msg_guild_id = msg.guild_id.unwrap_or_default();
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0) {
      if let Some(log) = ds.log {
        log.send_message(ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
              .title(&format!("{} purged messages from {:?}", msg.author.name, &users))
              .timestamp(chrono::Utc::now().to_rfc3339())
            })).await?;
      }
    }
  } else {
    channel_message(ctx, msg, "no messages found").await;
  }
  Ok(())
}

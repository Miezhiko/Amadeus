use crate::{
  collections::team::DISCORDS,
  common::{
    constants::MUTED_ROLE,
    msg::channel_message,
    help::channel::channel_by_name
  }
};

use std::num::NonZeroU64;

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateChannel, CreateEmbed, CreateEmbedFooter, CreateEmbedAuthor, GetMessages },
  model::{ id::{ ChannelId, UserId, RoleId, GuildId }
         , channel::*, Timestamp
         , permissions::Permissions },
  framework::standard::{ CommandResult
                       , macros::command
                       , Args }
};

const PURGE_ITERATIONS: usize = 10;

async fn mute_internal(ctx: &Context, guild_id: &GuildId, user_id: &UserId) -> anyhow::Result<()> {
  let guild = guild_id.to_partial_guild(&ctx).await?;
  let mut member = guild.member(&ctx, user_id).await?;
  if let Some(role) = guild.role_by_name(MUTED_ROLE) {
    if !member.roles.contains(&role.id) {
      if let Err(why) = member.add_role(&ctx, role).await {
        error!("Failed to assign muted role {why}");
      }
    }
  }
  Ok(())
}

async fn unmute_internal(ctx: &Context, guild_id: &GuildId, user_id: &UserId) -> anyhow::Result<()> {
  let guild = guild_id.to_partial_guild(&ctx).await?;
  let mut member = guild.member(&ctx, user_id).await?;
  if let Some(role) = guild.role_by_name(MUTED_ROLE) {
    if member.roles.contains(&role.id) {
      if let Err(why) = member.remove_role(&ctx, role).await {
        error!("Failed to unassign muted role {why}");
      }
    }
  }
  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (!msg.mentions.is_empty() && msg.mentions[0].bot) {
      if let Ok(user_id) = args.single::<u64>() {
        mute_internal(ctx, &guild_id, &UserId( to_nzu!(user_id) ) ).await?;
      } else {
        channel_message(ctx, msg, "you need to target who to mute").await;
      }
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      mute_internal(ctx, &guild_id, &target_user.id ).await?;
    }
  }
  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
async fn unmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (!msg.mentions.is_empty() && msg.mentions[0].bot) {
      if let Ok(user_id) = args.single::<u64>() {
        unmute_internal(ctx, &guild_id, &UserId( to_nzu!(user_id) ) ).await?;
      } else {
        channel_message(ctx, msg, "you need to target who to unmute").await;
      }
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      unmute_internal(ctx, &guild_id, &target_user.id ).await?;
    }
  }
  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn move_discussion(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ target_channel = ChannelId( args.single::<NonZeroU64>()? )
      , channel = target_channel.to_channel(ctx).await?
      , msg_guild_id = msg.guild_id.unwrap() };
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
    .send_message(ctx, CreateMessage::new()
        .content(comefrom_message)
    ).await?;

  channel_message(ctx, msg, &format!(
    "**{} suggested to move this discussion to {}**\n{}",
    &msg.author.name,
    target_channel.mention(),
    comefrom_message.link_ensured(ctx).await
  )).await;

  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn timeout(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ user_id = UserId( args.single::<NonZeroU64>()? )
      , msg_guild_id = msg.guild_id.unwrap() };

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
      guild.create_channel(&ctx.http, CreateChannel::new(channel_name.as_str())
                                  .permissions(permisssions_vec)
                                  .rate_limit_per_user(2*60) // seconds
                                  .kind(ChannelType::Text)
                                ).await?;
    let mut e = CreateEmbed::new()
      .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(msg.author.face()))
      .title(format!("You was timed out by {}", msg.author.name))
      .timestamp(chrono::Utc::now());
    if let Some(r) = reason {
      e = e.description(r);
    }
    timeout_channel.send_message(&ctx, CreateMessage::new()
      .content(&format!("{}\n", member.mention()))
      .embed(e)).await?;
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0.get()) {
      if let Some(log) = ds.log {
        log.send_message(ctx, CreateMessage::new()
          .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(&msg.author.face()))
            .title(&format!("{} timed out {}", msg.author.name, member.user.name))
            .timestamp(chrono::Utc::now())
            .footer(CreateEmbedFooter::new(&format!("~j {}", &timeout_channel.name)))
          )).await?;
      }
    }
  } else {
    return Err("User is not member of guild".into());
  }

  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn j(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ channel_name = args.single::<String>()?
      , msg_guild_id = msg.guild_id.unwrap() };
  if let Ok(channels) = msg_guild_id.channels(ctx).await {
    if let Some((channel, _)) = channel_by_name(ctx, &channels, &channel_name).await {
      let allow = Permissions::SEND_MESSAGES | Permissions::VIEW_CHANNEL;
      let overwrite_moderator = PermissionOverwrite {
        allow, deny: Permissions::empty(),
        kind: PermissionOverwriteType::Member(msg.author.id)
      };
      channel.create_permission(ctx, overwrite_moderator).await?;
      if let Err(why) = channel.send_message(ctx, CreateMessage::new()
                        .content(&format!("{} has joined", msg.author.name))).await {
        error!("Failed to log new user {why}");
      }
      if let Some(ds) = DISCORDS.get(&msg_guild_id.0.get()) {
        if let Some(log) = ds.log {
          log.send_message(ctx, CreateMessage::new()
            .embed(CreateEmbed::new()
              .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(&msg.author.face()))
              .title(&format!("{} joined {}", msg.author.name, &channel_name))
              .timestamp(chrono::Utc::now())
            )).await?;
        }
      }
    }
  }
  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
#[min_args(1)]
async fn untimeout(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  set!{ user_id = UserId( args.single::<NonZeroU64>()? )
      , msg_guild_id = msg.guild_id.unwrap() };

  let guild = msg_guild_id.to_partial_guild(ctx).await?;
  if let Ok(mut member) = guild.member(ctx, user_id).await {
    if let Some(muted_role) = guild.role_by_name(MUTED_ROLE) {
      if member.roles.contains(&muted_role.id) {
        member.remove_role(ctx, muted_role).await?;
      }
    }
    member.enable_communication(ctx).await?;
    msg.channel_id.send_message(&ctx, CreateMessage::new()
      .embed(CreateEmbed::new()
        .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(&msg.author.face()))
        .title(&format!("{} was untimeouted out by {}", member.user.name, msg.author.name))
        .timestamp(chrono::Utc::now())
      )).await?;
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0.get()) {
      if let Some(log) = ds.log {
        log.send_message(ctx, CreateMessage::new()
          .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(&msg.author.face()))
            .title(&format!("{} removed time out from {}", msg.author.name, member.user.name))
            .timestamp(chrono::Utc::now())
          )).await?;
      }
    }
  } else {
    return Err("User is not member of guild".into());
  }
  Ok(())
}

#[command]
#[bucket = "A"]
#[required_permissions(BAN_MEMBERS)]
async fn prison(ctx: &Context, msg: &Message) -> CommandResult {
  set!{ msg_guild_id  = msg.guild_id.unwrap()
      , guild         = msg_guild_id.to_partial_guild(ctx).await?
      , channels      = guild.channels(&ctx).await? };
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
#[bucket = "A"]
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
  let now = chrono::offset::Utc::now().date_naive();
  let mut last_msg_id_on_iteration = Some(msg.id);
  let mut messages = std::collections::HashSet::new();
  for _iteration in [0..PURGE_ITERATIONS] {
    if let Some(last_msg_id) = last_msg_id_on_iteration {
      if let Ok(msgs) = msg.channel_id.messages(ctx,
          GetMessages::default().before(last_msg_id).limit(255u8)
        ).await {
        let mut we_are_done = false;
        for message in &msgs {
          let diff = now - message.timestamp.date_naive();
          if diff.num_days() > 13 {
            we_are_done = true;
            break;
          }
          if users.iter().any(|u| *u == message.author.id.0.get()) {
            messages.insert(message.id);
          }
        }
        if we_are_done {
          break;
        }
        last_msg_id_on_iteration =
          msgs.last().map(|last_msg| last_msg.id);
      }
    }
  }

  if !messages.is_empty() {
    let messages_vec = Vec::from_iter(messages);
    msg.channel_id.delete_messages(ctx, messages_vec.as_slice()).await?;
    if let Err(why) = msg.delete(ctx).await {
      error!("Error deleting original command, {why}");
    }
    let msg_guild_id = msg.guild_id.unwrap();
    if let Some(ds) = DISCORDS.get(&msg_guild_id.0.get()) {
      if let Some(log) = ds.log {
        log.send_message(ctx, CreateMessage::new()
          .embed(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(&msg.author.name).icon_url(&msg.author.face()))
            .title(&format!("{} purged messages from {:?}", msg.author.name, &users))
            .timestamp(chrono::Utc::now())
          )).await?;
      }
    }
  } else {
    channel_message(ctx, msg, "no messages found").await;
  }
  Ok(())
}

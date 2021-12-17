use crate::common::{
  constants::MUTED_ROLE,
  msg::channel_message
};

use serenity::{
  model::{ id::ChannelId, channel::* },
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
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, target_user.id).await {
          if let Some(role) = guild.role_by_name(MUTED_ROLE) {
            if !member.roles.contains(&role.id) {
              if let Err(why) = member.add_role(&ctx, role).await {
                error!("Failed to assign muted role {:?}", why);
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
#[required_permissions(BAN_MEMBERS)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to unmute").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, target_user.id).await {
          if let Some(role) = guild.role_by_name(MUTED_ROLE) {
            if member.roles.contains(&role.id) {
              if let Err(why) = member.remove_role(&ctx, role).await {
                error!("Failed to unassign muted role {:?}", why);
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

use crate::{
  common::{ db::trees
          , msg::channel_message }
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{ CommandResult
                       , macros::command }
};

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn give_win(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target winner").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      let s = trees::add_win_points( guild_id.0
                                   , target_user.id.0
                                   ).await;
      let out = format!("win registered, {} wins in a row", s);
      channel_message(ctx, msg, &out).await;
    }
  }
  Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn register_lose(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target loser").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      let _ = trees::break_streak( guild_id.0
                                 , target_user.id.0
                                 ).await;
    }
  }
  Ok(())
}

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to mute").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, target_user.id).await {
          if let Some(role) = guild.role_by_name("muted") {
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
#[required_permissions(ADMINISTRATOR)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to unmute").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, target_user.id).await {
          if let Some(role) = guild.role_by_name("muted") {
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

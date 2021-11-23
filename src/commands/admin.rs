use crate::common::msg::channel_message;

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{ CommandResult
                       , macros::command }
};

#[command]
#[required_permissions(ADMINISTRATOR)]
async fn ban(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if msg.mentions.is_empty() || (msg.mentions.len() == 1 && msg.mentions[0].bot) {
      channel_message(ctx, msg, "you need to target who to ban").await;
    } else {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(member) = guild.member(&ctx, target_user.id).await {
          member.ban_with_reason(ctx, 0, &format!("banned by {}", msg.author.name)).await?;
        }
      }
    }
  }
  Ok(())
}

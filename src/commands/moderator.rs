use crate::common::{
    constants::MUTED_ROLE,
    msg::channel_message
  };
  
  use serenity::{
    model::channel::*,
    prelude::*,
    framework::standard::{ CommandResult
                         , macros::command
                         , Args }
  };
  
  use tokio::process::Command;
  
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
  async fn eix_update(ctx: &Context, msg: &Message) -> CommandResult {
    let _update = Command::new("zsh")
      .arg("-c")
      .arg("wupdate")
      .output()
      .await
      .expect("failed to run packages update");
    channel_message(ctx, msg, "update complete").await;
    Ok(())
  }
  
  #[command]
  #[required_permissions(BAN_MEMBERS)]
  #[min_args(1)]
  async fn eix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Ok(package) = args.single::<String>() {
      let eix_command = format!("eix {}", package);
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
  
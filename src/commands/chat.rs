use crate::{
  common::{
    points,
    msg::{ channel_message }
  },
  stains::ai::chain
};

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

#[command]
async fn score(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild) = msg.guild(&ctx).await {
    let (target, the_points) =
      if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
        let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
        if let Ok(p) = points::get_points( *guild.id.as_u64(), *target_user.id.as_u64()).await {
          ( &target_user.name, p )
        } else {
          ( &target_user.name, 0 )
        }
      } else if let Ok(p) = points::get_points( *guild.id.as_u64(), *msg.author.id.as_u64()).await {
          ( &msg.author.name, p )
        } else {
          ( &msg.author.name, 0 )
        };
    let out = format!("Score for {} : {}", target, the_points);
    let footer = format!("Requested by {}", msg.author.name);
    if let Err(why) = msg.channel_id.send_message(ctx, |m| m
      .embed(|e| e
      .description(out.as_str())
      .footer(|f| f.text(footer))
    )).await {
      error!("Failed to post score for {}, {:?}", target, why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)] // or 2?
async fn give(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild) = msg.guild(&ctx).await {
    if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
      let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
      if target_user.id == msg.author.id {
        channel_message(ctx, msg, "you don't give points to yourself").await;
      } else {
        let points_count = 
          if let Ok(first) = args.single::<u64>() {
            first
          } else if let Ok(second) = args.advance().single::<u64>() {
            second
          } else { 0 };
        if points_count > 0 {
          let (succ, rst) = points::give_points( *guild.id.as_u64()
                                               , *msg.author.id.as_u64()
                                               , *target_user.id.as_u64()
                                               , points_count).await;
          if succ {
            let out = format!("{} to {}", rst, target_user.name);
            if let Err(why) = msg.channel_id.send_message(ctx, |m| m
              .embed(|e| e
              .description(out.as_str())
              .footer(|f| f.text(msg.author.name.as_str()))
            )).await {
              error!("Failed to post give {:?}", why);
            }
          } else {
            channel_message(ctx, msg, rst.as_str()).await;
          }
        }
      }
    } else {
      channel_message(ctx, msg, "you need to target points reciever").await;
    };
  }
  Ok(())
}

#[command]
async fn quote(ctx: &Context, msg: &Message) -> CommandResult {
  if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
    let target = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
    if let Some(q) = chain::make_quote(ctx, msg, target.id, 9000).await {
      let footer = format!("Requested by {}", msg.author.name);
      if let Err(why) = msg.channel_id.send_message(ctx, |m| m
        .embed(|e| e
        .author(|a| a.icon_url(&target.face()).name(&target.name))
        .description(q)
        .footer(|f| f.text(footer))
      )).await {
        error!("Failed to quote {}, {:?}", target.name, why);
      }
    } else {
      let out = format!("No idea about {}", target.name);
      channel_message(ctx, msg, out.as_str()).await;
    }
  }
  Ok(())
}

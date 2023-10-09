use crate::common::{
  constants::PREFIX,
  db::trees::points,
  msg::channel_message
};

use crate::steins::ai::{ cache, chain, boris, uwu };

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateEmbed, CreateEmbedFooter, CreateEmbedAuthor },
  model::{ channel::*
         , guild::Member },
  framework::standard::{
    CommandResult, Args,
    macros::command
  }
};

#[command]
#[aliases(счёт, счет)]
#[description("displays user score")]
async fn score(ctx: &Context, msg: &Message) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    let (target, the_points) =
      if !(msg.mentions.is_empty() ||
           msg.mentions.len() == 1 && !msg.content.starts_with(PREFIX) && msg.mentions[0].bot) {
        let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
        if let Ok(p) = points::get_points( guild_id.get(), target_user.id.get() ).await {
          ( &target_user.name, p )
        } else {
          ( &target_user.name, 0 )
        }
      } else if let Ok(p) = points::get_points( guild_id.get(), msg.author.id.get() ).await {
          ( &msg.author.name, p )
        } else {
          ( &msg.author.name, 0 )
        };
    let out = format!("Score for {target}: {the_points}");
    let footer = format!("Requested by {}", msg.author.name);
    if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
      .embed(CreateEmbed::new()
      .description(&out)
      .footer(CreateEmbedFooter::new(footer))
    )).await {
      error!("Failed to post score for {target}, {why}");
    }
  }
  Ok(())
}

#[command]
#[description("displays ton N users by score")]
async fn top(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  let top_x =
    if let Ok(first) = args.single::<usize>() {
        if first > 99 {
          99
        } else {
          first
        }
      } else { 10 };

  let members =
    if let Some(guild) = msg.guild(ctx.cache.as_ref()) {
      guild.members.clone()
    } else {
      std::collections::HashMap::new()
    };

  let mut members_with_points: Vec<(Member, u64)> = Vec::new();
  if let Some(guild_id) = msg.guild_id {
    for (id, mem) in members {
      debug!("scanning points for {}", &mem.user.name);
      if let Ok(p) = points::get_points( guild_id.get(), id.get() ).await {
        members_with_points.push( (mem, p) );
      } else {
        members_with_points.push( (mem, 0) );
      }
    }
    members_with_points.sort_by(|(_, pa), (_, pb) | pa.cmp(pb));
    members_with_points.reverse();
    let mut out: Vec<String> = Vec::new();
    for (i, (m, p)) in members_with_points.iter().take(top_x).enumerate() {
      let n = i + 1;
      out.push(format!("{n}. **{}**: **{p}**", m.user.name));
    }
    let title = format!("Top {top_x} points");
    let footer = format!("Requested by {}", msg.author.name);
    if !out.is_empty() {
      if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
        .embed(CreateEmbed::new()
        .title(title)
        .description(out.join("\n"))
        .footer(CreateEmbedFooter::new(footer))
      )).await {
        error!("Failed to post top of users, {why}");
      }
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[bucket = "A"]
#[description("give mentioned user some own points")]
async fn give(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    if !(msg.mentions.is_empty() ||
         msg.mentions.len() == 1 && !msg.content.starts_with(PREFIX) && msg.mentions[0].bot) {
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
          let (succ, rst) = points::give_points( guild_id.get()
                                               , msg.author.id.get()
                                               , target_user.id.get()
                                               , points_count ).await;
          if succ {
            let out = format!("{rst} to {}", target_user.name);
            if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
              .embed(CreateEmbed::new()
              .description(&out)
              .footer(CreateEmbedFooter::new(&msg.author.name))
            )).await {
              error!("Failed to post give {why}");
            }
          } else {
            channel_message(ctx, msg, &rst).await;
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
#[aliases(цитата)]
#[bucket = "A"]
#[description("generate random quote of an user")]
async fn quote(ctx: &Context, msg: &Message) -> CommandResult {
  if !(msg.mentions.is_empty() || !msg.mentions.len() == 1 && msg.mentions[0].bot) {
    let target = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };
    if let Some(q) = chain::make_quote(ctx, msg, target.id).await {
      let footer = format!("Requested by {}", msg.author.name);
      if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
        .embed(CreateEmbed::new()
        .author(CreateEmbedAuthor::new(&target.name).icon_url(&target.face()))
        .description(q)
        .footer(CreateEmbedFooter::new(footer))
      )).await {
        error!("Failed to quote {}, {why}", target.name);
      }
    } else {
      channel_message( ctx
                     , msg
                     , &format!("No idea about {}", target.name)
                     ).await;
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[aliases(борис)]
#[description("metaphone for russian text")]
async fn boris(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  channel_message( ctx
                 , msg
                 , &boris::spell(args.message())
                 ).await;
  Ok(())
}

#[command]
#[min_args(1)]
#[description("uwu")]
async fn owo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  channel_message( ctx
                 , msg
                 , &uwu::spell(args.message())
                 ).await;
  Ok(())
}

#[command]
#[min_args(1)]
#[aliases(fem)]
#[description("feminize text")]
pub async fn feminize(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  {
    let kathoey = cache::KATHOEY.lock().await;
    channel_message( ctx
                   , msg
                   , &kathoey.feminize(args.message())
                   ).await;
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[aliases(ffem)]
#[description("feminize text with extreme mode!")]
async fn extreme_feminize(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  {
    let kathoey = cache::KATHOEY.lock().await;
    channel_message( ctx
                   , msg
                   , &kathoey.extreme_feminize(args.message())
                   ).await;
  }
  Ok(())
}

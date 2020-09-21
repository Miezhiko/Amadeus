use crate::{
  types::{ common::{ CoreGuild, CoreGuilds }
         , tracking::Bet },
  common::{ trees
          , msg::channel_message
  },
  steins::cyber
};

use serenity::{
  prelude::*,
  model::channel::*,
  model::id::UserId,
  model::guild::Member,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use regex::Regex;

async fn get_player(meme: &str, ctx: &Context, msg: &Message) -> eyre::Result<Member> {
  if meme.starts_with("<@") && meme.ends_with('>') {
    let re = Regex::new("[<@!>]").unwrap();
    let member_id = re.replace_all(&meme, "").into_owned();
    let member = msg.guild_id.unwrap().member(
      ctx, UserId(member_id.parse::<u64>().unwrap())).await;
    match member {
      Ok(m) => Ok(m.to_owned()),
      Err(why) => Err(eyre!(why))
    }
  } else {
    let guild = &msg.guild(ctx).await.unwrap();
    let member_name = meme.split('#').next().unwrap();
    for m in guild.members.values() {
      if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
        m.user.name == member_name
      {
        return Ok(m.clone())
      }
    }
    Err(eyre!("can't find this player"))
  }
}

#[command]
#[min_args(1)]
#[description("bet points on some player")]
async fn bet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
    let points_count = args.single::<u64>()?;
    if let Ok(p) = trees::get_points( guild_id.0, msg.author.id.0 ).await {
      if p < points_count {
        let err = format!("{} only has {}, need {}", msg.author.name, p, points_count);
        channel_message(ctx, msg, &err).await;
        return Ok(());
      }
      let mut games_lock = cyber::team_checker::GAMES.lock().await;
      for (_, track) in games_lock.iter_mut() {
        if track.still_live {
          for playa in &track.players {
            if playa.discord == meme.user.id.0 {
              let data = ctx.data.read().await;
              if let Some(core_guilds) = data.get::<CoreGuilds>() {
                let amadeus = core_guilds.get(&CoreGuild::Amadeus).unwrap();
                let bet = Bet { guild: guild_id.0
                              , member: msg.author.id.0
                              , points: points_count };
                let (succ, rst) = trees::give_points( guild_id.0
                                                    , msg.author.id.0
                                                    , *amadeus
                                                    , points_count ).await;
                if succ {
                  track.bets.push(bet);
                  let out = format!("bet **{}** on **{}**", points_count, meme.user.name);
                  let nickname_maybe =
                    if let Some(guild_id) = msg.guild_id {
                      msg.author.nick_in(&ctx, &guild_id).await
                    } else { None };
                  let nick = nickname_maybe.unwrap_or_else(|| msg.author.name.clone());
                  if let Err(why) = msg.channel_id.send_message(ctx, |m| m
                    .embed(|e| e
                    .description(&out)
                    .color(0xed9e2f)
                    .author(|a| a.icon_url(&msg.author.face()).name(&nick))
                  )).await {
                    error!("Failed to post give {:?}", why);
                  }
                } else {
                  channel_message(ctx, msg, &rst).await;
                }
                return Ok(())
              }
            }
          }
        }
      }
      channel_message(ctx, msg, "No running games for this player").await;
    }
  }
  Ok(())
}

use crate::{
  types::{ serenity::{ CoreGuild, CoreGuilds }
         , tracking::Bet },
  common::{ db::trees::points
          , help::members::get_player
          , msg::channel_message
  },
  steins::warcraft
};

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateEmbed, CreateEmbedAuthor },
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

#[command]
#[min_args(1)]
#[description("bet points on some player")]
async fn bet(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Some(guild_id) = msg.guild_id {
    let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
    let points_count = args.single::<u64>()?;
    let p = points::get_points( guild_id.0.get(), msg.author.id.0.get() ).await?;
    if p < points_count {
      let err = format!("{} only has {p}, need {points_count}", msg.author.name);
      channel_message(ctx, msg, &err).await;
      return Ok(());
    }
    let mut games_lock = warcraft::poller::GAMES.lock().await;
    for (_, track) in games_lock.iter_mut() {
      if track.still_live {
        for playa in &track.players {
          if playa.player.discord == meme.user.id.0.get() {
            if track.bets.iter().any(|b| b.member == msg.author.id.0.get()) {
              channel_message(ctx, msg, "you already have bet on this match").await;
              return Ok(());
            }
            let mut amadeus_guild = None;
            { // trying to hold ctx data for minimum time
              let data = ctx.data.read().await;
              if let Some(core_guilds) = data.get::<CoreGuilds>() {
                if let Some(amadeus) = core_guilds.get(&CoreGuild::Amadeus) {
                  amadeus_guild = Some(*amadeus);
                }
              }
            }
            if let Some(amadeus) = amadeus_guild {
              let bet = Bet { guild: guild_id.0.get()
                            , member: msg.author.id.0.get()
                            , points: points_count
                            , positive: true
                            , registered: false };
              let (succ, rst) = points::give_points( guild_id.0.get()
                                                    , msg.author.id.0.get()
                                                    , amadeus
                                                    , points_count ).await;
              if succ {
                track.bets.push(bet);
                if let Err(why) = msg.delete(&ctx).await {
                  error!("Error deleting original command {why}");
                }
                let out = format!("bets **{points_count}** on **{}**", meme.user.name);
                let nickname_maybe =
                  if let Some(guild_id) = msg.guild_id {
                    msg.author.nick_in(&ctx, &guild_id).await
                  } else { None };
                let nick = nickname_maybe.unwrap_or_else(|| msg.author.name.clone());
                // not really sure if there should be response on bet
                if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
                  .embed(CreateEmbed::new()
                  .description(&out)
                  .color(0xed3e7fu32)
                  .author(CreateEmbedAuthor::new(&nick).icon_url(&msg.author.face()))
                )).await {
                  error!("Failed to post bet {why}");
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
  Ok(())
}

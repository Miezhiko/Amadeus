use std::sync::Arc;
use typemap::Key;

use serenity::{
  client::bridge::gateway::{ShardId, ShardManager},
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult,
    macros::command
  },
};

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

#[command]
pub fn help(ctx: &mut Context, msg: &Message) -> CommandResult {
  let version = format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
    .embed(|e| e
      .title("Amadeus")
      .url("https://github.com/Qeenon/Amadeus")
      .image("https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .description("memory storage and artificial intelligence system")
      .fields(vec![
        ("Age", "18", true),
        ("Birthdate", "July 25th", true),
        ("Blood Type", "A", true)
        ])
      .fields(vec![
        ("Height", "160 cm", true),
        ("Weight", "45 kg", true),
        ("Version", version.as_str(), true)
        ])
      .field("user commands", "help: shows this
quote <@user>: something from that user...
ping: shows shard latency", false)
      .field("music commands", "join: to music channel
leave: from music channel
play <url>: play an radio stream or youtube music
repeat: plays last stream again", false)
      .field("warcraft commands", "today: show tournaments today (same with tomorrow or yesterday)
weekends: show tours at weekend
stats <nick>: shows ladder race stats (nick - battle tag or tag without numbers) (without nick will use discord name)
ongoing: show ongoing solo matches
lineup <title>: | map player map2 player2+player3 (title is optional)", false)
      .footer(|f| f.text("As explained by Maho Hiyajo, Amadeus' capabilities of storing memory data make it possible to back-up the data of a person's memories. This could be useful, for example, to fight against memory loss."))
      .colour((246, 111, 0)))) {
    error!("Error sending help message: {:?}", why);
  }
  Ok(())
}

#[command]
pub fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
  // The shard manager is an interface for mutating, stopping, restarting, and
  // retrieving information about shards.
  let data = ctx.data.read();
  let shard_manager = match data.get::<ShardManagerContainer>() {
    Some(v) => v,
    None => {
      let _ = msg.reply(&ctx, "There was a problem getting the shard manager");
      return Ok(());
    },
  };
  set! { manager = shard_manager.lock()
       , runners = manager.runners.lock() };
  // Shards are backed by a "shard runner" responsible for processing events
  // over the shard, so we'll get the information about the shard runner for
  // the shard this command was sent over.
  let runner = match runners.get(&ShardId(ctx.shard_id)) {
    Some(runner) => runner,
    None => {
      let _ = msg.reply(&ctx,  "No shard found");
      return Ok(());
    },
  };
  let _ = msg.reply(&ctx, &format!("The shard latency is {:?}", runner.latency));
  Ok(())
}

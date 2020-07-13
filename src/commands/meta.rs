use std::sync::Arc;

use serenity::{
  client::bridge::gateway::{ShardId, ShardManager},
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use qrcode::{
  QrCode,
  render::unicode,
};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

#[command]
#[aliases(commands)]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
  let version = format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
    .embed(|e| e
      .title("Amadeus")
      .url("https://github.com/Qeenon/Amadeus")
      .image(
"https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
      .thumbnail(
"https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .description(
"__**memory storage and artificial intelligence system**__
to execute commands use ~*<command>* or @Amadeus *<command>*
note: replace *<thing>* in help with text without < > brackets")
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
      .field("user commands",
"• **help**: shows this
• **quote** *<@user>*: something from that user
• **score** *<@user>*: show user points
• **embed** *<title>* *<description>*: create embed
• **qrcode** *<something>*: creates QR code
• **urban** *<thing>*: explains a thing
• **ping**: shows shard latency", false)
      .field("music commands",
"• **join**: to your voice channel (you should be there)
• **leave**: from voice channel
• **play** *<url>*: play an radio stream or youtube music
• **repeat**: plays last stream again", false)
      .field("warcraft commands",
"• **today**: show tournaments today (same with tomorrow or yesterday)
• **weekends**: show tours at weekend
• **stats** *<nick>*: shows ladder race stats (nick - battle tag or tag without numbers) (without nick will use discord name)
• **ongoing**: show ongoing solo matches
• **lineup** *<title>: | map player map2 player2+player3* (title is optional)", false)
      .footer(|f| f.text(
"As explained by Maho Hiyajo, Amadeus' capabilities of storing memory data make it possible to back-up the data of a person's memories. This could be useful, for example, to fight against memory loss."))
      .colour((246, 111, 0)))).await {
    error!("Error sending help message: {:?}", why);
  }
  Ok(())
}

#[command]
async fn embed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  set!{ title = args.single::<String>()?
      , description = args.single::<String>()? };
  msg.channel_id.send_message(&ctx.http, |m|
    m.embed(|e| e.title(title)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .description(description)
    )
  ).await?;
  Ok(())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  // The shard manager is an interface for mutating, stopping, restarting, and
  // retrieving information about shards.
  let data = ctx.data.read().await;
  let shard_manager = match data.get::<ShardManagerContainer>() {
    Some(v) => v,
    None => {
      msg.reply(&ctx, "There was a problem getting the shard manager").await?;
      return Ok(());
    },
  };
  set! { manager = shard_manager.lock().await
       , runners = manager.runners.lock().await };
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
  
  msg.reply(&ctx, &format!("The shard latency is {:?}", runner.latency)).await?;
  Ok(())
}

// Struct used to deserialize the output of the urban dictionary api call...
#[derive(Deserialize, Clone)]
struct UrbanDict {
  definition: String,
  permalink: String,
  thumbs_up: u32,
  thumbs_down: u32,
  author: String,
  written_on: String,
  example: String,
  word: String,
}

#[command]
async fn qrcode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let words = args.message();
  let code = QrCode::new(words).unwrap();
  let image = code.render::<unicode::Dense1x2>()
      .dark_color(unicode::Dense1x2::Light)
      .light_color(unicode::Dense1x2::Dark)
      .build();
  msg.channel_id.say(ctx, format!("```{}```", image)).await?;
  Ok(())
}

#[command]
async fn urban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let term = args.message();
  let url = reqwest::Url::parse_with_params("http://api.urbandictionary.com/v0/define", &[("term", term)])?;

  let reqwest = reqwest::Client::new();
  let resp = reqwest.get(url)
      .send().await?.json::<Vec<UrbanDict>>().await?;

  if resp.is_empty() {
    msg.channel_id.say(ctx, format!("The term '{}' has no Urban Definitions", term)).await?;
  } else {
    let choice = &resp[0];
    let parsed_definition = &choice.definition.replace("[", "").replace("]", "");
    let parsed_example = &choice.example.replace("[", "").replace("]", "");
    let mut fields = vec![
      ("Definition", parsed_definition, false),
    ];
    if parsed_example != &"".to_string() {
      fields.push(("Example", parsed_example, false));
    }
    if let Err(why) = msg.channel_id.send_message(ctx, |m| {
      m.embed(|e| {
        e.title(&choice.word);
        e.url(&choice.permalink);
        e.description(
          format!("submitted by **{}**\n\n:thumbsup: **{}** ┇ **{}** :thumbsdown:\n",
                      &choice.author, &choice.thumbs_up, &choice.thumbs_down));
        e.fields(fields);
        e.timestamp(choice.clone().written_on);
        e
      });
      m
    }).await {
      if "Embed too large." == why.to_string() {
        msg.channel_id.say(ctx, &choice.permalink).await?;
      } else {
        msg.channel_id.say(ctx, why).await?;
      }
    };
  }
  Ok(())
}

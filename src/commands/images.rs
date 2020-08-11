use crate::types::common::PubCreds;

use serenity::{
  prelude::*,
  model::prelude::*,
  framework::standard::{
    CommandResult,
    macros::command, Args
  }
};

use std::{ collections::HashMap, sync::Arc };
use reqwest::Url;
use serde::Deserialize;
use rand::{ prelude::StdRng, Rng, SeedableRng };
use reqwest::Client as Reqwest;

pub struct ReqwestClient;
impl TypeMapKey for ReqwestClient {
  type Value = Arc<Reqwest>;
}

#[derive(Debug, Deserialize)]
struct Response {
  results: Vec<GifResult>
}

#[derive(Debug, Deserialize)]
struct GifResult {
  url: String,
  media: Vec<HashMap<String, Media>>
}

#[derive(Debug, Deserialize)]
struct Media {
  url: String
}

async fn fetch_gifs(ctx: &Context, search: &str, amount: u32, filter: &str)
        -> Result<Vec<GifResult>, Box<dyn std::error::Error + Send + Sync>> {

  set!{ data            = ctx.data.read().await
      , reqwest_client  = data.get::<ReqwestClient>().unwrap()
      , tenor_key       = data.get::<PubCreds>().unwrap().get("tenor").unwrap().as_str() };

  let url = Url::parse_with_params("https://api.tenor.com/v1/search",
            &[ ("q", search)
             , ("key", tenor_key)
             , ("limit", &format!("{}", amount))
             , ("contentfilter", filter)
             ])?;

  let resp = reqwest_client.get(url)
                           .send().await?
                           .json::<Response>().await?;

  Ok(resp.results)
}

#[command]
async fn hug(ctx: &Context, msg: &Message) -> CommandResult {
  if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };

    let gifs = fetch_gifs(ctx, "hug anime", 50, "off ").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 49);

    msg.channel_id.send_message(ctx, |m|
      m.embed(|e| e.color(0xed9e2f)
                   .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                   .description(format!("hugs {}", target_user.name))
                   .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  } else {
    msg.channel_id.say(ctx, "You want to give a hug? Please mention who you want to hug!").await?;
  }
  Ok(())
}

#[command]
async fn pat(ctx: &Context, msg: &Message) -> CommandResult {
  if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };

    let gifs = fetch_gifs(ctx, "pat anime", 50, "off").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 49);

    msg.channel_id.send_message(ctx, |m|
      m.embed(|e| e.color(0x27e6d9)
                   .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                   .description(format!("pats {}", target_user.name))
                  .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  } else {
    msg.channel_id.say(ctx, "I wanna pat someone! Please mention who to pat!").await?;
  }
  Ok(())
}

#[command]
async fn slap(ctx: &Context, msg: &Message) -> CommandResult {
  if !msg.mentions.is_empty() && !(msg.mentions.len() == 1 && msg.mentions[0].bot) {
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] } else { &msg.mentions[0] };

    let gifs = fetch_gifs(ctx, "slap anime", 50, "off").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 49);

    msg.channel_id.send_message(ctx, |m|
      m.embed(|e| e.color(0xd62929)
                   .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                   .description(format!("slaps {}", target_user.name))
                   .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  } else {
    msg.channel_id.say(ctx, "Wait... who do I slap again? Please mention the person!").await?;
  }
  Ok(())
}

#[command]
async fn wave(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "wave anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);

  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} waves", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn sex(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let filter = 
      if msg.channel(ctx).await.unwrap().is_nsfw() {
        "off" 
      } else {
        "low"
      };
  let gifs = fetch_gifs(ctx, "sex anime", 50, filter).await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn ahegao(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let filter = 
      if msg.channel(ctx).await.unwrap().is_nsfw() {
        "off" 
      } else {
        "low"
      };
  let gifs = fetch_gifs(ctx, "ahegao", 50, filter).await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn dance(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "dancing anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn clap(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "clap anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn lol(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "lol anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xa656e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn angry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "angry anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is angry", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn sad(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "sad anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is sad", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn happy(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "happy anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is happy", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn shrug(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "shrug anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} shrugs", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn shock(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "sock anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is shocked", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn nervous(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "nervous anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is feeling nervous", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn confused(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "confused anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} confused", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn cry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "cry anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} is crying", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn cringe(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "cringe", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x3252e3)
                .author(|a| a.icon_url(&msg.author.face()).name( &msg.author.name ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn annoyed(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "annoyed anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn omg(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "omg anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn smile(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "smile anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);

  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xafb2e3)
                .author(|a| a.icon_url(&msg.author.face()).name( format!("{} smiles", &msg.author.name) ))
                .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn ew(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "ew anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn awkward(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "awkward anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn oops(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "oops anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn lazy(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "lazy anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn hungry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "hungry anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn srtessed(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "srtessed anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn scared(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "scared anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn bored(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "bored anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn yes(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "yes anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn no(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "no anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn bye(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "bye anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn sorry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "sorry anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn sleepy(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "sleepy anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn wink(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "wink anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
async fn whatever(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "whatever anime", 50, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 49);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0xaf52e3)
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

#[command]
#[aliases(gif)]
async fn gifsearch(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if args.is_empty() {
    msg.channel_id.say(ctx, "Please provide a search string after the command!").await?;
    return Ok(())
  }
  let search_string = args.message();
  let filter = 
      if msg.channel(ctx).await.unwrap().is_nsfw() {
        "off" 
      } else {
        "low"
      };
  let gifs = fetch_gifs(ctx, search_string, 10, filter).await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 9);
  msg.channel_id.send_message(ctx, |m|
    m.embed(|e| e.color(0x5ed13b)
                 .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
  Ok(())
}

/*
 * Inpired by: https://github.com/bdashore3/CourtJester/blob/serenity/src/commands/images.rs
 */

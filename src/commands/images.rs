use serenity::{
  prelude::*,
  model::prelude::*,
  framework::standard::{
    CommandResult,
    macros::command, Args
  }
};

use std::{collections::HashMap, sync::Arc};
use reqwest::Url;
use serde::Deserialize;
use rand::{prelude::StdRng, Rng, SeedableRng};
use reqwest::Client as Reqwest;

pub struct ReqwestClient;
impl TypeMapKey for ReqwestClient {
  type Value = Arc<Reqwest>;
}

pub struct PubCreds;
impl TypeMapKey for PubCreds {
  type Value = Arc<HashMap<String, String>>;
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

    let gifs = fetch_gifs(ctx, "hug anime", 100, "off ").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 99);

    msg.channel_id.send_message(ctx, |m| {
      m.embed(|e| {
        e.color(0xed9e2f);
        e.description(format!("{} hugs {}", msg.author.name, target_user.name));
        e.image(&gifs[val].media[0].get("gif").unwrap().url);
        e
      })
    }).await?;
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

    let gifs = fetch_gifs(ctx, "pat anime", 100, "off").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 99);

    msg.channel_id.send_message(ctx, |m| {
      m.embed(|e| {
        e.color(0x27e6d9);
        e.description(format!("{} pats {}", msg.author.name, target_user.name));
        e.image(&gifs[val].media[0].get("gif").unwrap().url);
        e
      })
    }).await?;
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

    let gifs = fetch_gifs(ctx, "slap anime", 100, "off").await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 99);

    msg.channel_id.send_message(ctx, |m| {
      m.embed(|e| {
        e.color(0xd62929);
        e.description(format!("{} slaps {}", msg.author.name, target_user.name));
        e.image(&gifs[val].media[0].get("gif").unwrap().url);
        e
      })
    }).await?;
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
  let gifs = fetch_gifs(ctx, "wave anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.description(format!("{} waves", msg.author.name));
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;

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
  let gifs = fetch_gifs(ctx, "sex anime", 100, filter).await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
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
  let gifs = fetch_gifs(ctx, "ahegao", 100, filter).await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

#[command]
async fn clap(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "clap anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

#[command]
async fn lol(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "lol anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

#[command]
async fn angry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "angry anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.description(format!("{} is angry", msg.author.name));
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

#[command]
async fn shrug(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "shrug anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.description(format!("{} shrugs", msg.author.name));
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

#[command]
async fn cry(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "cry anime", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x3252e3);
      e.description(format!("{} is crying!", msg.author.name));
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;

  Ok(())
}

#[command]
async fn cringe(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let gifs = fetch_gifs(ctx, "cringe", 100, "off").await?;
  let mut rng = StdRng::from_entropy();
  let val = rng.gen_range(0, 99);

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x634644);
      e.description(format!("{} thinks that's cringey", msg.author.name));
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;

  Ok(())
}

#[command]
#[aliases("gif")]
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
  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| {
      e.color(0x5ed13b);
      e.image(&gifs[val].media[0].get("gif").unwrap().url);
      e
    })
  }).await?;
  Ok(())
}

/*
 * Inpired by: https://github.com/bdashore3/CourtJester/blob/serenity/src/commands/images.rs
 */

use crate::types::common::{ ReqwestClient, PubCreds };

use serenity::{
  prelude::*,
  model::prelude::*,
  framework::standard::{
    CommandResult,
    macros::command, Args
  },
  utils::Colour
};

use std::{ collections::HashMap };
use reqwest::Url;
use serde::Deserialize;
use rand::{ prelude::StdRng, Rng, SeedableRng };

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

enum GType {
  Own(String),
  Target(String),
  Nothing
}

fn own(x: &str) -> GType { GType::Own(String::from(x)) }
fn target(x: &str) -> GType { GType::Target(String::from(x)) }

async fn gifx<C: Into<Colour>>( ctx: &Context
                              , msg: &Message
                              , fetch: &str
                              , color: C
                              , target: GType
                              , nsfw: bool
                              ) -> CommandResult {
  if match target {
    GType::Target(_) => !(msg.mentions.is_empty() || (!msg.mentions.is_empty() && msg.mentions[0].bot)),
    GType::Own(_) => true,
    GType::Nothing => true
  } {
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command {:?}", why);
    }
    let filter = if nsfw {
        if msg.channel(ctx).await.unwrap().is_nsfw() {
          "off" 
        } else {
          "low"
        }
      } else  { "off" };

    let gifs = fetch_gifs(ctx, fetch, 50, filter).await?;
    let mut rng = StdRng::from_entropy();
    let val = rng.gen_range(0, 49);

    let nickname_maybe =
      if let Some(guild_id) = msg.guild_id {
        msg.author.nick_in(&ctx, &guild_id).await
      } else { None };

    let nick = nickname_maybe.unwrap_or_else(|| msg.author.name.clone());

    match target {
      GType::Target(t) => {
        let target_user = if msg.mentions.len() > 1 { &msg.mentions[1] }
                                               else { &msg.mentions[0] };
        msg.channel_id.send_message(ctx, |m|
          m.embed(|e| e.color(color)
                      .author(|a| a.icon_url(&msg.author.face()).name(&nick))
                      .description(format!("{} {}", t, target_user.name))
                      .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
      },
      GType::Own(o) => {
        msg.channel_id.send_message(ctx, |m|
          m.embed(|e| e.color(color)
                      .author(|a| a.icon_url(&msg.author.face()).name(&nick))
                      .description(o)
                      .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
      },
      GType::Nothing => {
        msg.channel_id.send_message(ctx, |m|
          m.embed(|e| e.color(color)
                       .author(|a| a.icon_url(&msg.author.face()).name(&nick))
                       .image(&gifs[val].media[0].get("gif").unwrap().url))).await?;
      }
    }
  } else {
    msg.channel_id.say(ctx, "Please mention a person!").await?;
  }
  Ok(())
}

#[command]
async fn hug(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "hug anime"
      , 0xed9e2f
      , target("hugs")
      , false ).await
}

#[command]
async fn pat(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "pat anime"
      , 0x27e6d9
      , target("pats")
      , false ).await
}

#[command]
async fn slap(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "slap anime"
      , 0xd62929
      , target("slaps")
      , false ).await
}

#[command]
async fn wave(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "wave anime"
      , 0x3252e3
      , own("waves")
      , false ).await
}

#[command]
async fn sex(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "sex anime"
      , 0x3252f3
      , GType::Nothing
      , true ).await
}

#[command]
async fn ahegao(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "ahegao anime"
      , 0xf252c1
      , GType::Nothing
      , true ).await
}

#[command]
async fn dance(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "dancing anime"
      , 0x22b2c1
      , GType::Nothing
      , false ).await
}

#[command]
async fn clap(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "clap anime"
      , 0x22c2c1
      , GType::Nothing
      , false ).await
}

#[command]
async fn lol(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "lol anime"
      , 0xa656e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn angry(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "angry anime"
      , 0x424203
      , own("Angry!")
      , false ).await
}

#[command]
async fn sad(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "sad anime"
      , 0x3252e3
      , own("sad :(")
      , false ).await
}

#[command]
async fn happy(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "happy anime"
      , 0x6252e3
      , own("happy :)")
      , false ).await
}

#[command]
async fn shrug(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "shrug anime"
      , 0x3252e3
      , own(r"shrugs ¯\_(ツ)_/¯")
      , false ).await
}

#[command]
async fn shock(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "shock anime"
      , 0x722223
      , GType::Nothing
      , false ).await
}

#[command]
async fn nervous(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "nervous anime"
      , 0x3252e3
      , own("feeling nervous")
      , false ).await
}

#[command]
async fn confused(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "confused anime"
      , 0x3292e3
      , own("confused")
      , false ).await
}

#[command]
async fn cry(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "cry anime"
      , 0x126223
      , own("crying")
      , false ).await
}

#[command]
async fn pout(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "pout anime"
      , 0x3252e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn cringe(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "cringe"
      , 0x111111
      , GType::Nothing
      , true ).await
}

#[command]
async fn annoyed(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "annoyed anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn omg(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "omg anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn smile(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "smile anime"
      , 0xafb2e3
      , own("smiles")
      , false ).await
}

#[command]
async fn smug(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "smug anime"
      , 0xaf2213
      , GType::Nothing
      , false ).await
}

#[command]
async fn ew(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "ew anime"
      , 0xaf82e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn awkward(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "awkward anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}


#[command]
async fn oops(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "oops anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn lazy(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "lazy anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn hungry(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "hungry anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn stressed(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "stressed anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn scared(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "scared anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn bored(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "bored anime"
      , 0xad52c3
      , GType::Nothing
      , false ).await
}

#[command]
async fn yes(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "yes anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn no(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "no anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn bye(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "bye anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn sorry(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "sorry anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn sleepy(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "sleepy anime"
      , 0x3f22a3
      , GType::Nothing
      , false ).await
}

#[command]
async fn wink(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "wink anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn facepalm(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "facepalm anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
async fn whatever(ctx: &Context, msg: &Message) -> CommandResult {
  gifx( ctx, msg
      , "whatever anime"
      , 0xaf52e3
      , GType::Nothing
      , false ).await
}

#[command]
#[aliases(gif)]
#[description("search for specific gif")]
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

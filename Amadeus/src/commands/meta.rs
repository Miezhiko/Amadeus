use crate::{
  types::serenity::ReqwestClient,
  common::{
    i18n::{ help_i18n, RU },
    system::stats::*
  }
};

use serenity::{
  builder::{ CreateMessage, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, EditMessage },
  prelude::*,
  model::{
    channel::*,
    Timestamp
  },
  framework::standard::{
    CommandResult, Args,
    macros::command
  }
};

use tokio::process::Command;

use rand::Rng;
use regex::Regex;
use once_cell::sync::Lazy;

use qrcode::{
  QrCode,
  render::unicode,
};

use chrono::Timelike;

#[command]
#[aliases(версия)]
#[description = "shows current version"]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
  let version = format!("Version {}", env!("CARGO_PKG_VERSION"));
  if let Err(why) = msg.channel_id.send_message(&ctx, CreateMessage::new()
    .embed(CreateEmbed::new()
      .title("Amadeus")
      .description(&version)
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    )
   ).await {
    error!("Error sending version message, {why}");
  }
  Ok(())
}

#[command]
#[aliases(помощь)]
async fn help_ru(ctx: &Context, msg: &Message) -> CommandResult {
  help_i18n(ctx, msg, &RU).await;
  Ok(())
}

#[command]
#[min_args(2)]
async fn embed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let nickname_maybe =
    if let Some(guild_id) = msg.guild_id {
      msg.author.nick_in(&ctx, &guild_id).await
    } else { None };
  set!{ title = args.single::<String>()?
      , description = args.rest()
      , nick  = nickname_maybe.unwrap_or_else(|| msg.author.name.clone())
      , red   = rand::thread_rng().gen_range(0..255)
      , green = rand::thread_rng().gen_range(0..255)
      , blue  = rand::thread_rng().gen_range(0..255) };
  msg.channel_id.send_message(&ctx.http, CreateMessage::new()
    .embed(CreateEmbed::new()
      .title(title)
      .colour((red, green, blue))
      .author(CreateEmbedAuthor::new(&nick).icon_url(&msg.author.face()))
      .description(description)
    )
  ).await?;
  Ok(())
}

#[command]
async fn qrcode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let words = args.message();
  if let Ok(code) = QrCode::new(words) {
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    msg.channel_id.say(ctx, format!("```{image}```")).await?;
  } else {
    msg.channel_id.say(ctx, "failed to get qr code").await?;
  }
  Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Definition {
  definition: String,
  permalink: String,
  thumbs_up: u64,
  sound_urls: Vec<String>,
  author: String,
  word: String,
  defid: u64,
  current_vote: String,
  written_on: String,
  example: String,
  thumbs_down: u64
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
  list: Vec<Definition>
}

#[command]
#[description("Find term in Urban Dictionary")]
#[min_args(1)]
async fn urban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let term = args.message();
  let reqwest_client = {
    set!{ data            = ctx.data.read().await
        , reqwest_client  = data.get::<ReqwestClient>().unwrap() };
    reqwest_client.clone()
  };

  let url = reqwest::Url::parse_with_params
    ("http://api.urbandictionary.com/v0/define", &[("term", term)])?;

  let resp = reqwest_client.get(url)
      .send().await?.json::<ApiResponse>().await?;

  if resp.list.is_empty() {
    msg.channel_id.say(ctx, format!("The term '{term}' has no Urban Definitions")).await?;
  } else {
    let choice = &resp.list[0];
    let parsed_definition = &choice.definition.replace(['[', ']'], "");
    let parsed_example = &choice.example.replace(['[', ']'], "");
    let mut fields = vec![
      ("Definition", parsed_definition.as_str(), false),
    ];
    if parsed_example != &"".to_string() {
      fields.push(("Example", parsed_example.as_str(), false));
    }
    let footer = format!("Requested by {}", msg.author.name);
    let timestamp: Timestamp = choice.written_on.parse()?;
    if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
      .embed(CreateEmbed::new()
        .title(&choice.word)
        .url(&choice.permalink)
        .description(
          format!("submitted by **{}**\n\n:thumbsup: **{}** ┇ **{}** :thumbsdown:\n",
                     &choice.author, &choice.thumbs_up, &choice.thumbs_down))
        .fields(fields)
        .timestamp(timestamp)
        .footer(CreateEmbedFooter::new(footer))
      )
    ).await {
      if "Embed too large." == why.to_string() {
        msg.channel_id.say(ctx, &choice.permalink).await?;
      } else {
        msg.channel_id.say(ctx, &format!("Error: {why}")).await?;
      }
    };
  }
  Ok(())
}

#[command]
#[aliases(about)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }

  let (_, uptime_string) = get_uptime("Uptime:  ").await;

  set!{ guild_count   = ctx.cache.guilds().len()
      , channel_count = ctx.cache.guild_channel_count()
      , user_count    = ctx.cache.user_count()
      , sys_info      = get_system_info(ctx).await
      , footer = format!("Requested by {}", msg.author.name) };

  let eb = CreateEmbed::new()
    .title(format!("Amadeus {}", env!("CARGO_PKG_VERSION")))
    .color(0xf51010u32)
    .description(format!(
"```
Servers:  {}
Channels: {}
Users:    {}
Memory:   {}
Salieri:  {}
Database: {}
Latency:  {}
{}
```", guild_count
    , channel_count
    , user_count
    , sys_info.memory
    , sys_info.memory_saliery
    , sys_info.db_size
    , sys_info.shard_latency
    , uptime_string ))
    .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    .footer(CreateEmbedFooter::new(footer));

  msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(eb)
  ).await?;

  Ok(())
}

#[command]
async fn changelog(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }

  let git_log = Command::new("sh")
        .arg("-c")
        .arg("git --no-pager log -n 13")
        .output()
        .await
        .expect("failed to execute process");
  if let Ok(git_log_stdout) = &String::from_utf8(git_log.stdout) {
    static RE1: Lazy<Regex> =
      Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
    static RE2: Lazy<Regex> =
      Lazy::new(|| Regex::new(r"Date.*").unwrap());
    let mut descr = RE1.replace_all(git_log_stdout, "").to_string();
    descr = RE2.replace_all(&descr, "").to_string();
    descr = descr.lines()
                 .filter(|l| !l.trim().is_empty())
                 .collect::<Vec<&str>>()
                 .join("\n");
    descr = descr.replace("commit", "**commit**");
    descr = descr.replace("Author:", "*author:*");

    let footer = format!("Requested by {}", msg.author.name);

    let eb = CreateEmbed::new()
      .color(0x13fac1u32)
      .title("Changelog")
      .description(descr)
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .footer(CreateEmbedFooter::new(footer));

    msg.channel_id.send_message(ctx, CreateMessage::new()
      .embed(eb)
    ).await?;
  }

  Ok(())
}

#[command]
#[description("display shard uptime since last restart")]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let footer = format!("Requested by {}", msg.author.name);
  let (start_time, uptime_string) = get_uptime("uptime").await;

  let eb = CreateEmbed::new()
    .color(0xe535ccu32)
    .title(uptime_string)
    .description(format!("start time: {start_time}"))
    .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    .footer(CreateEmbedFooter::new(footer));

  msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(eb)
  ).await?;

  Ok(())
}

async fn time_internal(msg: &Message, args: Args) -> anyhow::Result<CreateEmbed> {
  let mb_tz = args.message();

  let utc = chrono::Utc::now();
  let time_format = "%k:%M";

  set!{ cet_time = utc.with_timezone(&chrono_tz::CET).time()
      , msk_time = utc.with_timezone(&chrono_tz::Europe::Moscow).time()
      , est_time = utc.with_timezone(&chrono_tz::EST).time() };

  set!{ cet = cet_time.format(time_format)
      , msk = msk_time.format(time_format)
      , est = est_time.format(time_format) };

  set!{ munutes_first_half = cet_time.minute() < 30
      , cet_pattern = (cet_time.hour12().1, munutes_first_half)
      , msk_pattern = (msk_time.hour12().1, munutes_first_half)
      , est_pattern = (est_time.hour12().1, munutes_first_half) };

  let get_emoji = |pattern: (u32, bool)| -> char {
    match pattern {
      (1, true)   => '🕐',
      (1, false)  => '🕜',
      (2, true)   => '🕑',
      (2, false)  => '🕝',
      (3, true)   => '🕒',
      (3, false)  => '🕞',
      (4, true)   => '🕓',
      (4, false)  => '🕟',
      (5, true)   => '🕔',
      (5, false)  => '🕠',
      (6, true)   => '🕕',
      (6, false)  => '🕡',
      (7, true)   => '🕖',
      (7, false)  => '🕢',
      (8, true)   => '🕗',
      (8, false)  => '🕣',
      (9, true)   => '🕘',
      (9, false)  => '🕣',
      (10, true)  => '🕙',
      (10, false) => '🕣',
      (11, true)  => '🕚',
      (11, false) => '🕦',
      (12, true)  => '🕛',
      (12, false) => '🕧',
      _           => '?'
    }
  };

  set!{ cet_emoji = get_emoji(cet_pattern)
      , msk_emoji = get_emoji(msk_pattern)
      , est_emoji = get_emoji(est_pattern) };

  let mut desc =
    format!("**CET**: {cet} {cet_emoji}\n**MSK**: {msk} {msk_emoji}\n**EST**: {est} {est_emoji}");

  if let Ok(tz) = mb_tz.parse::<chrono_tz::Tz>() {
    let tz_time = utc.with_timezone(&tz).time();
    let tz_fmt = tz_time.format(time_format);
    let tz_pattern = (est_time.hour12().1, munutes_first_half);
    let tz_emoji = get_emoji(tz_pattern);
    desc = format!("{desc}\n**{mb_tz}**: {tz_fmt} {tz_emoji}");
  }

  let mut eb = CreateEmbed::new()
                .color(0xe735cc)
                .title("Time")
                .description(desc)
                .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");

  if !msg.author.bot {
    let footer = format!("Requested by {}", msg.author.name);
    eb = eb.footer(CreateEmbedFooter::new(footer));
  }

  Ok(eb)
}

#[command]
#[aliases(время)]
#[description("display current time")]
pub async fn time(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }

  let eb = time_internal(msg, args).await?;

  msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(eb)
  ).await?;
  Ok(())
}

pub async fn time_slash(ctx: &Context, msg: &mut Message, args: Args) -> anyhow::Result<()> {
  let eb = time_internal(msg, args).await?;

  msg.edit(ctx, EditMessage::default()
    .content("")
    .embed(eb)
  ).await?;

  Ok(())
}

use crate::{
  types::common::ReqwestClient,
  types::options::ROptions,
  common::{
    msg::{ direct_message, reply },
    i18n::{ help_i18n, RU },
    options
  },
  steins::gate::behavior::START_TIME
};

use std::sync::Arc;

use serenity::{
  builder::CreateEmbed,
  client::{
    bridge::gateway::{ShardId, ShardManager},
    bridge::voice::ClientVoiceManager
  },
  prelude::*,
  model::{
    misc::Mentionable,
    id::GuildId,
    id::ChannelId,
    channel::*
  },
  framework::standard::{
    CommandResult, Args,
    macros::command,
    Delimiter
  },
  voice
};

use chrono::{ Duration, Utc };
use tokio::process::Command;

use rand::Rng;
use regex::Regex;

use qrcode::{
  QrCode,
  render::unicode,
};

use chrono::Timelike;

pub struct VoiceManager;
pub struct ShardManagerContainer;

impl TypeMapKey for VoiceManager {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

#[command]
#[aliases(версия)]
#[description = "shows current version"]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
  let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
    .embed(|e| e
      .title("Amadeus")
      .description(&version)
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    )
   ).await {
    error!("Error sending version message: {:?}", why);
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
    error!("Error deleting original command {:?}", why);
  }
  let nickname_maybe =
    if let Some(guild_id) = msg.guild_id {
      msg.author.nick_in(&ctx, &guild_id).await
    } else { None };
  set!{ title = args.single::<String>()?
      , description = args.rest()
      , nick  = nickname_maybe.unwrap_or_else(|| msg.author.name.clone())
      , red   = rand::thread_rng().gen_range(0, 255)
      , green = rand::thread_rng().gen_range(0, 255)
      , blue  = rand::thread_rng().gen_range(0, 255) };
  msg.channel_id.send_message(&ctx.http, |m|
    m.embed(|e| e.title(title)
                 .colour((red, green, blue))
                 .author(|a| a.icon_url(&msg.author.face()).name(&nick))
                 .description(description)
    )
  ).await?;
  Ok(())
}

#[command]
async fn qrcode(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let words = args.message();
  let code = QrCode::new(words).unwrap();
  let image = code.render::<unicode::Dense1x2>()
      .dark_color(unicode::Dense1x2::Light)
      .light_color(unicode::Dense1x2::Dark)
      .build();
  msg.channel_id.say(ctx, format!("```{}```", image)).await?;
  Ok(())
}

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
    error!("Error deleting original command {:?}", why);
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
    msg.channel_id.say(ctx, format!("The term '{}' has no Urban Definitions", term)).await?;
  } else {
    let choice = &resp.list[0];
    let parsed_definition = &choice.definition.replace("[", "").replace("]", "");
    let parsed_example = &choice.example.replace("[", "").replace("]", "");
    let mut fields = vec![
      ("Definition", parsed_definition, false),
    ];
    if parsed_example != &"".to_string() {
      fields.push(("Example", parsed_example, false));
    }
    let footer = format!("Requested by {}", msg.author.name);
    if let Err(why) = msg.channel_id.send_message(ctx, |m| {
      m.embed(|e|
        e.title(&choice.word)
         .url(&choice.permalink)
         .description(
           format!("submitted by **{}**\n\n:thumbsup: **{}** ┇ **{}** :thumbsdown:\n",
                      &choice.author, &choice.thumbs_up, &choice.thumbs_down))
         .fields(fields)
         .timestamp(choice.clone().written_on)
         .footer(|f| f.text(footer))
      );
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

#[derive(Default, Debug)]
struct SysInfo {
  pub shard_latency: String,
  pub memory: String,
  pub db_size: String
}

async fn get_system_info(ctx: &Context) -> SysInfo {
  let data = ctx.data.read().await;
  let mut sys_info = SysInfo::default();
  sys_info.shard_latency = {
    set! { shard_manager = data.get::<ShardManagerContainer>().unwrap()
         , manager       = shard_manager.lock().await
         , runners       = manager.runners.lock().await
         , runner_raw    = runners.get(&ShardId(ctx.shard_id)) };
    match runner_raw {
      Some(runner) => {
        match runner.latency {
          Some(ms) => format!("{}ms", ms.as_millis()),
          None => "?ms".to_string()
        }
      },
      None => "?ms".to_string()
    }
  };
  let pid = std::process::id().to_string();
  let mem_stdout = Command::new("sh")
          .arg("-c")
          .arg(&format!("pmap {} | tail -n 1 | awk '/[0-9]K/{{print $2}}'", &pid))
          .output()
          .await
          .expect("failed to execute process");
  if let Ok(mem_used) = &String::from_utf8(mem_stdout.stdout) {
    let memory_mb = mem_used[..mem_used.len() - 2].parse::<f32>().unwrap()/1024f32;
    sys_info.memory = if memory_mb >= 1024.0 {
      let memory_gb = memory_mb / 1024f32;
      format!("{:.3} GB", memory_gb)
      } else { format!("{:.3} MB", memory_mb) };
  } else {
    error!("Failed to parse mem stdout");
  }
  let dbs_stdout = Command::new("sh")
          .arg("-c")
          .arg("du trees/tree.lusf | cut -f 1")
          .output()
          .await
          .expect("failed to execute process");
  if let Ok(db_size_str) = &String::from_utf8(dbs_stdout.stdout) {
    if let Ok(db_kb) = db_size_str[..db_size_str.len() - 1].parse::<u32>() {
      sys_info.db_size = if db_kb >= 1024 {
        let db_mb = db_kb as f32 / 1024f32;
        format!("{:.3} MB", db_mb)
        } else { format!("{:.3} KB", db_kb) };
    } else {
      error!("Failed to parse: {}", db_size_str);
      sys_info.db_size = String::from("?");
    }
  } else {
    error!("Failed to parse du stdout");
  }
  sys_info
}

async fn get_uptime(start: &str) -> (String, String) {
  let nao = Utc::now();
  let start_time = START_TIME.lock().await;
  let since_start_time : Duration = nao - *start_time;
  let mut uptime_string = String::from(start);

  let dd = since_start_time.num_days();
  if dd > 0 {
    uptime_string = format!("{} {}d", uptime_string, dd);
  }
  let hh = since_start_time.num_hours() - dd*24;
  if hh > 0 {
    uptime_string = format!("{} {}h", uptime_string, hh);
    if dd == 0 {
      let mm = since_start_time.num_minutes() - hh*60;
      uptime_string = format!("{} {}m", uptime_string, mm);
    }
  } else {
    let mm = since_start_time.num_minutes();
    if mm > 0 {
      uptime_string = format!("{} {}m {}s", uptime_string, mm
                                          , since_start_time.num_seconds() - mm*60);
    } else {
      uptime_string = format!("{} {}s", uptime_string, since_start_time.num_seconds());
    }
  }

  ( start_time.format("%Y %b %d %H:%M").to_string(), uptime_string )
}

#[command]
#[aliases(about)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }

  let mut eb = CreateEmbed::default();
  let (_, uptime_string) = get_uptime("Uptime:  ").await;

  set!{ guild_count   = ctx.cache.guilds().await.len()
      , channel_count = ctx.cache.guild_channel_count().await
      , user_count    = ctx.cache.user_count().await
      , sys_info      = get_system_info(ctx).await
      , footer = format!("Requested by {}", msg.author.name) };

  eb.title(format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string()));
  eb.color(0xf51010);
  eb.description(format!(
"```
Servers:  {}
Channels: {}
Users:    {}
Memory:   {}
Database: {}
Latency:  {}
{}
```", guild_count, channel_count, user_count, sys_info.memory, sys_info.db_size, sys_info.shard_latency
    , uptime_string ));
  eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
  eb.footer(|f| f.text(footer));

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| { e.0 = eb.0; e })
  }).await?;

  Ok(())
}

#[command]
async fn changelog(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }

  let git_log = Command::new("sh")
        .arg("-c")
        .arg("git --no-pager log -n 13")
        .output()
        .await
        .expect("failed to execute process");
  if let Ok(git_log_stdout) = &String::from_utf8(git_log.stdout) {

    lazy_static! {
      static ref RE1: Regex = Regex::new(r"<(.*?)>").unwrap();
      static ref RE2: Regex = Regex::new(r"Date.*").unwrap();
    }
    let mut descr = RE1.replace_all(&git_log_stdout, "").to_string();
    descr = RE2.replace_all(&descr, "").to_string();
    descr = descr.lines()
                 .filter(|l| !l.trim().is_empty())
                 .collect::<Vec<&str>>()
                 .join("\n");
    descr = descr.replace("commit", "**commit**");
    descr = descr.replace("Author:", "*author:*");

    let mut eb = CreateEmbed::default();
    let footer = format!("Requested by {}", msg.author.name);

    eb.color(0x13fac1);
    eb.title("Changelog");

    eb.description(descr);

    eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
    eb.footer(|f| f.text(footer));

    msg.channel_id.send_message(ctx, |m| {
      m.embed(|e| { e.0 = eb.0; e })
    }).await?;
  }

  Ok(())
}

#[command]
#[description("display shard uptime since last restart")]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let mut eb = CreateEmbed::default();
  let footer = format!("Requested by {}", msg.author.name);

  let (start_time, uptime_string) = get_uptime("uptime").await;

  eb.color(0xe535cc);
  eb.title(uptime_string);
  eb.description(format!("start time: {}", start_time));
  eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
  eb.footer(|f| f.text(footer));

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| { e.0 = eb.0; e })
  }).await?;

  Ok(())
}

pub async fn rejoin_voice_channel(ctx : &Context, conf: &ROptions) {
  if conf.rejoin && conf.last_guild != 0 && conf.last_channel != 0 {
    set!{ last_guild_conf   = GuildId( conf.last_guild )
        , last_channel_conf = ChannelId( conf.last_channel ) };
    let manager_lock =
      ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;
    if manager.join(last_guild_conf, last_channel_conf).is_some() {
      info!("Rejoined voice channel: {}", last_channel_conf);
      if conf.last_stream != "" {
        if let Some(handler) = manager.get_mut(last_guild_conf) {
          match voice::ytdl(&conf.last_stream).await {
            Ok(source) => handler.play(source),
            Err(why)   => error!("Err starting source: {:?}", why)
          };
        }
      }
    } else {
      error!("Failed to rejoin voice channel: {}", last_channel_conf);
    }
  }
}

#[command]
#[description("join voice channel")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
  let guild = match msg.guild(&ctx).await {
    Some(guild) => guild,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    }
  };
  let guild_id = guild.id;
  let channel_id = guild
    .voice_states.get(&msg.author.id)
    .and_then(|voice_state| voice_state.channel_id);
  let connect_to = match channel_id {
    Some(channel) => channel,
    None => {
      reply(ctx, msg, "You're not in a voice channel").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
    .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if manager.join(guild_id, connect_to).is_some() {
    let mut opts = options::get_roptions().await?;
    if opts.last_guild != guild_id.0
    || opts.last_channel != connect_to.0
    || !opts.rejoin {
      opts.rejoin = true;
      opts.last_guild = guild_id.0;
      opts.last_channel = connect_to.0;
      options::put_roptions(&opts).await?;
    }
    if let Err(why) = msg.channel_id.say(&ctx, &format!("I've joined {}", connect_to.mention())).await {
      error!("failed to say joined {:?}", why);
    }
  } else {
    direct_message(ctx, msg, "Some error joining the channel...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("leave voice channel")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    },
  };
  let manager_lock = ctx.data.read()
      .await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  let has_handler = manager.get(guild_id).is_some();
  if has_handler {
    manager.remove(guild_id);
    let _ = msg.channel_id.say(&ctx, "I left voice channel");
    let mut conf = options::get_roptions().await?;
    if conf.rejoin {
      conf.rejoin = false;
      options::put_roptions(&conf).await?;
    }
  } else {
    reply(ctx, &msg, "I'm not in a voice channel").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("play some mp3/youtube stream")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let url =
    if !args.is_empty() {
      match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
          reply(ctx, msg, "You must provide a URL to a video or audio").await;
          return Ok(());
        }
      }
    } else {
      let conf = options::get_roptions().await?;
      conf.last_stream
    };
  if !url.starts_with("http") {
    reply(ctx, msg, "You must provide a valid URL").await;
    return Ok(());
  }
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      reply(ctx, msg, "Error finding channel info...").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
      .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if let Some(handler) = manager.get_mut(guild_id) {
    let source = match voice::ytdl(&url).await {
      Ok(source) => source,
      Err(why) => {
        error!("Err starting source: {:?}", why);
        reply(ctx, msg, &format!("Sorry, error sourcing ffmpeg {:?}", why)).await;
        return Ok(());
      }
    };
    handler.play_only(source);
    let mut conf = options::get_roptions().await?;
    let last_stream_conf = conf.last_stream;
    if last_stream_conf != url {
      conf.last_stream = url.clone();
      options::put_roptions(&conf).await?;
    }
    reply(ctx, msg, &format!("playing stream: {}", url)).await;
  } else {
    reply(ctx, msg, "Not in a voice channel to play in...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("play last stream again")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
  play(ctx, msg, Args::new("", &[Delimiter::Single(' ')])).await
}

#[command]
#[aliases(время)]
#[description("display current time")]
async fn time(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }

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

  let mut eb = CreateEmbed::default();
  let footer = format!("Requested by {}", msg.author.name);
  eb.color(0xe735cc);
  eb.title("Time");
  eb.description(format!("**CET**: {} {}\n**MSK**: {} {}\n**EST**: {} {}", cet, cet_emoji, msk, msk_emoji, est, est_emoji));
  eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
  eb.footer(|f| f.text(footer));

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| { e.0 = eb.0; e })
  }).await?;
  Ok(())
}

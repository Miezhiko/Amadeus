use crate::stains::gate::behavior::START_TIME;

use std::sync::Arc;

use serenity::{
  builder::CreateEmbed,
  client::bridge::gateway::{ShardId, ShardManager},
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use chrono::{ Duration, Utc };
use tokio::process::Command;

use rand::{
  Rng
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
#[aliases(версия)]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
  let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
    .embed(|e| e
      .title("Amadeus")
      .description(version.as_str())
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    )
   ).await {
    error!("Error sending version message: {:?}", why);
  }
  Ok(())
}

#[command]
#[aliases(commands)]
// Note: not fully sure what to do when that list will be too big to show (and how to test it)
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
to execute commands use `~<command>` or `@Amadeus <command>`, replace `<thing>` in help with text without `< >` brackets")
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
"• **ttt** *<@user>* *<N>*: play tic tac toe for points
• **quote** *<@user>*: something from that *user*
• **score** *<@user>*: show *user* points
• **top** *<N>*: shows top N users
• **give** *<@user>* *<N>*: give *user* your *N* points
• **embed** *<title>* *<description>*: create embed
• **qrcode** *<something>*: creates QR code
• **urban** *<thing>*: explains a thing
• **gif** *<thing>*, cry, hug, pat, slap, cringe, wave, sex, ahegao, clap, shrug, lol, angry, dance, confused, shock, nervous, sad, happy, annoyed, omg, smile, ew, awkward, oops, lazy, hungry, srtessed, scared, bored, yes, no, bye, sorry, sleepy, wink, whatever
• **ru2en** *<text>*: translation, also **en2ru** **en2de** **en2fr** **de2en** **fr2en**"
, false)
      .field("music commands",
"• **join**: to your voice channel (you should be there)
• **leave**: from voice channel
• **play** *<url>*: play an radio stream or youtube music
• **repeat**: plays last stream again", false)
      .field("warcraft commands",
"• **today**: show tournaments today (same with **tomorrow**, **yesterday** or **weekends**)
• **stats** *<nick>*: shows ladder race stats (nick - battle tag or tag without numbers) (without nick will use discord name)
• **ongoing**: show ongoing solo matches
• **lineup** *<title> | map player map2 player2+player3* (title is optional)", false)
      .footer(|f| f.text(
"As explained by Maho Hiyajo, Amadeus' capabilities of storing memory data make it possible to back-up the data of a person's memories. This could be useful, for example, to fight against memory loss."))
      .colour((246, 111, 0)))).await {
    error!("Error sending help message: {:?}", why);
  }
  Ok(())
}

#[command]
#[aliases(помощь)]
async fn help_ru(ctx: &Context, msg: &Message) -> CommandResult {
  let version = format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
    .embed(|e| e
      .title("Амадеус")
      .url("https://github.com/Qeenon/Amadeus")
      .image(
"https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
      .thumbnail(
"https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .description(
"__**Система хранения памяти и искусственного интеллекта**__
Используйте `~<команда>` или `@Amadeus <команда>`, замените `<параметр>` на своё сообщение без `< >`, оно нужно для доп. параметров (Вместо «<параметр>» может быть другое)")
      .fields(vec![
        ("Возраст", "18", true),
        ("День рождения", "July 25th", true),
        ("Группа крови", "A", true)
        ])
      .fields(vec![
        ("Рост", "160 cm", true),
        ("Вес", "45 kg", true),
        ("Версия", version.as_str(), true)
        ])
      .field("Пользовательские команды",
"• **Крестики_нолики** *<@пользователь>* *<N>*: Игра - камень, ножницы, бумага на очки
• **Цитата** *<@пользователь>*: Случайная цитата *пользователя*
• **Счёт** *<@пользователь>*: Посмотреть счёт *пользователя*
• **top** *<N>*: Топ 10 *пользователей* по очкам
• **give** *<@пользователь>* *<N>*: Дать *пользователю* *N*(ое) - количество очков
• **embed** *<Заголовок>* *<Описание>*: Создание выделенного сообщения
• **qrcode** *<Сообщение>*: Создание QR кода
• **urban** *<Термин>*: Опеределение термина взятого из «urban»
• **gif** *<параметр>*, cry, hug, pat, slap, cringe, wave, sex, ahegao, clap, shrug, lol, angry, dance, confused, shock, nervous, sad, happy, annoyed, omg, smile, ew, awkward, oops, lazy, hungry, srtessed, scared, bored, yes, no, bye, sorry, sleepy, wink, whatever
• **ru2en** *<text>*: переводчик, также **en2ru** **en2de** **en2fr** **de2en** **fr2en**"
, false)
      .field("Музыкальные команды",
"• **join**: Подключение к каналу (в котором вы находитесь)
• **leave**: Выход из канала
• **play** *<url>*: Включить радиопоток или музыку на YouTube
• **repeat**: повтор последней *url*", false)
      .field("Warcraft команды",
"• **Сегодня**: Показать турниры сегодня (Тоже самое для **Завтра**, **Вчера** или **Weekends**)
• **Статистика** *<Никнейм>*: Показывает статистику ладдера (Никнейм - battle tag или tag но без номера)(Без этого будет использовать имя дискорда)
• **ongoing**: Матчи которые идут прямо сейчас в под ладдере
• **lineup** *<title> | карта игроков карта2 игрок2+игрок3* (Название не обезательно)", false)
      .footer(|f| f.text(
"Как обьяснила Хияджо Махо, возможности Амадеуса по хранению данных в памяти позволяет создать резервную копию воспоминаний человека. Это может быть полезно, например, для борьбы с потерей памяти"))
      .colour((246, 111, 0)))).await {
    error!("Error sending help message: {:?}", why);
  }
  Ok(())
}

#[command]
#[min_args(2)]
async fn embed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  set!{ title = args.single::<String>()?
      , description = args.rest()
      , red   = rand::thread_rng().gen_range(0, 255)
      , green = rand::thread_rng().gen_range(0, 255)
      , blue  = rand::thread_rng().gen_range(0, 255) };
  msg.channel_id.send_message(&ctx.http, |m|
    m.embed(|e| e.title(title)
                 .colour((red, green, blue))
                 .author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name))
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
async fn urban(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let term = args.message();
  let url = reqwest::Url::parse_with_params
    ("http://api.urbandictionary.com/v0/define", &[("term", term)])?;

  let reqwest = reqwest::Client::new();
  let resp = reqwest.get(url)
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
  pub memory: String
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
          .arg(format!("pmap {} | tail -n 1 | awk '/[0-9]K/{{print $2}}'", &pid).as_str())
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
  sys_info
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }

  let mut eb = CreateEmbed::default();

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
Latency:  {}
```", guild_count, channel_count, user_count, sys_info.memory, sys_info.shard_latency));
  eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
  eb.footer(|f| f.text(footer));

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| { e.0 = eb.0; e })
  }).await?;

  Ok(())
}

#[command]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let mut eb = CreateEmbed::default();
  let footer = format!("Requested by {}", msg.author.name);

  let nao = Utc::now();
  let start_time = START_TIME.lock().await;
  let since_start_time : Duration = nao - *start_time;
  let mut uptime_string = String::from("uptime");

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

  eb.color(0xe535cc);
  eb.title(uptime_string);
  eb.description(format!("start time: {}", start_time.to_string()));
  eb.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
  eb.footer(|f| f.text(footer));

  msg.channel_id.send_message(ctx, |m| {
    m.embed(|e| { e.0 = eb.0; e })
  }).await?;

  Ok(())
}

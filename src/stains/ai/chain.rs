use crate::{
  types::common::AllGuilds,
  common::{
    help::lang,
    msg::{ reply, channel_message }
  },
  collections::base::{ CONFUSION, CONFUSION_RU, OBFUSCATION, OBFUSCATION_RU },
  collections::channels::AI_LEARN,
  stains::ai::{ boris, uwu, bert }
};

use serenity::{
  prelude::*,
  model::{ channel::{ Message }
         , id::{ GuildId, UserId, ChannelId }
         , channel::GuildChannel
         , gateway::Activity }
};

use regex::Regex;

use markov::Chain;

use rand::{
  Rng
};

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use chrono::{ Duration, Utc, DateTime };
use tokio::sync::{ Mutex, MutexGuard };

// Note: 15000 is known to be safe value
// But I'm not sure what's maximal supported by discord limit
static CACHE_MAX: u64 = 15_666;

// Note: machine learning based translation is very hard without cuda
// even 5 from each channel can take hours on high server load
// but it's sometimes and I can't understand why exactly it's happening
static TRANSLATION_MAX: u32 = 10;

// Note: use 66 for low activity/comfortable behavior
pub static ACTIVITY_LEVEL: AtomicU32 = AtomicU32::new(50);

lazy_static! {
  pub static ref CACHE_ENG: Mutex<Chain<String>>    = Mutex::new(Chain::new());
  pub static ref CACHE_ENG_STR: Mutex<Vec<String>>  = Mutex::new(Vec::new());
  pub static ref CACHE_RU: Mutex<Chain<String>>     = Mutex::new(Chain::new());
  pub static ref LAST_UPDATE: Mutex<DateTime<Utc>>  = Mutex::new(Utc::now());
}

pub async fn update_cache(ctx: &Context, channels: &HashMap<ChannelId, GuildChannel>) {
  info!("updating AI chain has started");
  ctx.set_activity(Activity::listening("Updating chain")).await;
  ctx.idle().await;

  setm!{ cache_eng = CACHE_ENG.lock().await
       , cache_ru = CACHE_RU.lock().await
       , cache_eng_str = CACHE_ENG_STR.lock().await };

  if !cache_eng.is_empty() || !cache_ru.is_empty() || !cache_eng_str.is_empty() {
    *cache_eng = Chain::new();
    *cache_ru = Chain::new();
    cache_eng_str.clear();
  }
  let mut ru_messages_for_translation : Vec<String> = vec![];
  let re1 = Regex::new(r"<(.*?)>").unwrap();
  let re2 = Regex::new(r":(.*?):").unwrap();
  for chan in channels.keys() {
    if let Some(c_name) = chan.name(&ctx).await {
      if AI_LEARN.iter().any(|c| c == &c_name) {
        if let Ok(messages) = chan.messages(&ctx, |r|
          r.limit(CACHE_MAX)
        ).await {
          trace!("updating ai chain from {}", &c_name);
          let mut i : u32 = 0;
          for mmm in messages {
            if !mmm.author.bot && !mmm.content.starts_with('~') {
              let is_to_bot = !mmm.mentions.is_empty() && (&mmm.mentions).iter().any(|u| u.bot);
              if !is_to_bot {
                let mut result_string = re1.replace_all(&mmm.content, "").to_string();
                result_string = re2.replace_all(&result_string, "").to_string();
                result_string = result_string.replace(": ", "");
                let is_http = result_string.starts_with("http") && !result_string.starts_with("https://images");
                let result = result_string.trim();
                if !result.is_empty() && !result.contains('$') && !is_http {
                  if lang::is_russian(result) {
                    cache_ru.feed_str(result);
                    if i < TRANSLATION_MAX {
                      ru_messages_for_translation.push(result.to_string());
                      i += 1;
                    }
                  } else {
                    cache_eng.feed_str(result);
                    cache_eng_str.push(result.to_string());
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  for confuse in CONFUSION_RU.iter() {
    cache_ru.feed_str( confuse );
  }
  for confuse in CONFUSION.iter() {
    cache_eng.feed_str( confuse );
  }
  // Translate cache_ru for big cache_eng_str
  // It's long operation so we run it 5 seonds later as separated task
  ctx.set_activity(Activity::listening("Translating cache")).await;
  let ctx_clone = ctx.clone();
  tokio::spawn(async move {
    tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
    if let Ok(mut translated) = bert::ru2en_many(ru_messages_for_translation).await {
      cache_eng_str.append(&mut translated);
    }
    ctx_clone.set_activity(Activity::listening("Update complete")).await;
  });
  ctx.online().await;
  info!("Updating cache complete");
}

pub async fn actualize_cache(ctx: &Context, guild_id: &GuildId) {
  let nao = Utc::now();
  let mut last_update = LAST_UPDATE.lock().await;
  let since_last_update : Duration = nao - *last_update;
  if since_last_update > Duration::hours(2) {
    let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
    set!{ data       = ctx.data.read().await
        , server_ids = data.get::<AllGuilds>().unwrap() };
    let servers = server_ids.iter()
                            .map(|x64| GuildId(*x64))
                            .collect::<Vec<GuildId>>();
    for server in servers {
      if let Ok(serv_channels) = server.channels(ctx).await {
        all_channels.extend(serv_channels);
      }
    }
    if let Ok(channels) = guild_id.channels(ctx).await {
      all_channels.extend(channels);
    }
    update_cache(ctx, &all_channels).await;
    *last_update = nao;
  }
}

pub async fn make_quote(ctx: &Context, msg : &Message, author_id: UserId, limit: u64) -> Option<String> {
  let mut have_something = false;
  if let Some(guild) = msg.guild(&ctx).await {
    let mut chain = Chain::new();
    let re1 = Regex::new(r"<(.*?)>").unwrap();
    let re2 = Regex::new(r":(.*?):").unwrap();
    let guild_id = guild.id;
    if let Ok(channels) = guild_id.channels(&ctx).await {
      for (chan, _) in channels {
        if let Some(c_name) = chan.name(&ctx).await {
          if AI_LEARN.iter().any(|c| c == &c_name) {
            if let Ok(messages) = chan.messages(&ctx, |r|
              r.limit(limit)
            ).await {
              for mmm in messages {
                if mmm.author.id == author_id && !mmm.content.starts_with('~') {
                  let mut result_string = re1.replace_all(&mmm.content, "").to_string();
                  result_string = re2.replace_all(&result_string, "").to_string();
                  result_string = result_string.replace(": ", "");
                  let is_http = result_string.starts_with("http") && !result_string.starts_with("https://images");
                  let result = result_string.trim();
                  if !result.is_empty() && !result.contains('$') && !is_http {
                    chain.feed_str(result);
                    if !have_something {
                      have_something = true;
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    if have_something {
      return Some(chain.generate_str());
    }
  }
  None
}

pub async fn generate_with_language(ctx: &Context, guild_id: &GuildId, russian : bool) -> String {
  actualize_cache(ctx, guild_id).await;
  let chain : MutexGuard<Chain<String>> =
    if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  chain.generate_str()
}

pub async fn generate_english_or_russian(ctx: &Context, guild_id: &GuildId) -> String {
  let rndx = rand::thread_rng().gen_range(0, 2);
  generate_with_language(&ctx, &guild_id, rndx != 1).await
}

pub async fn generate(ctx: &Context, msg: &Message, mbrussian: Option<bool>) -> String {
  let mut out = String::new();
  if let Some(guild) = msg.guild(&ctx).await {
    set!{ msg_content = &msg.content
        , guild_id = guild.id };
    let russian = if let Some(rus) = mbrussian
      { rus } else { lang::is_russian(msg_content) };
    actualize_cache(ctx, &guild_id).await;
    let chain : MutexGuard<Chain<String>> =
    if russian {
        CACHE_RU.lock().await
      } else {
        CACHE_ENG.lock().await
      };
    out = chain.generate_str();
    let rndx = rand::thread_rng().gen_range(0, 13);
    if rndx == 1 {
      if russian {
        out = boris::spell(&out);
      } else {
        out = uwu::spell(&out);
      }
    }
  }
  out
}

pub fn obfuscate(msg_content : &str) -> String {
  let mut chain = Chain::new();
  let russian = lang::is_russian(msg_content);
  if !russian {
    for confuse in OBFUSCATION.iter() {
      chain.feed_str( confuse );
    }
  } else {
    for confuse in OBFUSCATION_RU.iter() {
      chain.feed_str( confuse );
    }
  }
  chain.feed_str(msg_content);
  let rndx = rand::thread_rng().gen_range(0, 2);
  let cahin_string = chain.generate_str();
  if rndx == 1 {
    if russian {
      boris::spell(&cahin_string)
    } else {
      uwu::spell(&cahin_string)
    }
  } else {
    cahin_string
  }
}

pub async fn response(ctx: &Context, msg : &Message) {
  set!{ msg_content = &msg.content
      , russian = lang::is_russian(msg_content) };
  let rndx : u32 = rand::thread_rng().gen_range(0, 15);
  if rndx == 1 {
    let answer = generate(&ctx, &msg, Some(russian)).await;
    if !answer.is_empty() {
      reply(&ctx, &msg, &answer).await;
    }
  } else {
    let text = if russian {
      if let Ok(translated) = bert::ru2en(String::from(msg_content)).await {
        translated
      } else { String::from(msg_content) }
      } else { String::from(msg_content) };
    let mut response =
      if msg_content.ends_with('?') {
        if let Ok(answer) = bert::ask(text).await {
          answer
        } else { String::new() }
      } else if let Ok(answer) = bert::chat(text, *msg.author.id.as_u64()).await {
        answer
      } else { String::new() };
    if russian {
      if let Ok(translated) = bert::en2ru(response.clone()).await {
        response = translated
      }
    }
    reply(&ctx, &msg, &response).await;
  }
}

pub async fn chat(ctx: &Context, msg : &Message) {
  let russian = lang::is_russian(&msg.content);
  let rndx : u32 = rand::thread_rng().gen_range(0, 9);
  let mut bert_generated = false;
  let mut answer =
    if rndx == 1 {
      let text = if russian {
        if let Ok(translated) = bert::ru2en(msg.content.clone()).await {
          translated
        } else { msg.content.clone() }
        } else { msg.content.clone() };
      if msg.content.ends_with('?') {
        if let Ok(answer) = bert::ask(text).await {
          bert_generated = true;
          answer
        } else {
          generate(&ctx, &msg, Some(russian)).await
        }
      } else if let Ok(answer) = bert::chat(text, *msg.author.id.as_u64()).await {
        bert_generated = true;
        answer
      } else {
        generate(&ctx, &msg, Some(russian)).await
      }
    } else {
      generate(&ctx, &msg, Some(russian)).await
    };
  if rndx == 1 && bert_generated && russian {
    if let Ok(translated) = bert::en2ru(answer.clone()).await {
      answer = translated;
    }
  }
  if !answer.is_empty() {
    let rnd = rand::thread_rng().gen_range(0, 3);
    if rnd == 1 {
      reply(&ctx, &msg, &answer).await;
    } else {
      channel_message(&ctx, &msg, &answer).await;
    }
  }
}

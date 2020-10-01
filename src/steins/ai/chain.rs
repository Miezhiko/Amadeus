use crate::{
  types::common::{ AllGuilds, ChannelLanguage },
  common::{ help::lang
          , msg::{ reply, channel_message }
          , trees::{ register, check_registration }
  },
  collections::base::{ CONFUSION
                     , CONFUSION_RU
                     , OBFUSCATION
                     , OBFUSCATION_RU },
  collections::channels::AI_LEARN,
  steins::ai::{ boris, uwu, bert },
  handler::RESTORE
};

use serenity::{
  prelude::*,
  model::{ channel::Message
         , id::{ GuildId, UserId, ChannelId }
         , channel::GuildChannel
         , gateway::Activity },
  futures::StreamExt
};

use std::sync::atomic::Ordering;
use async_std::fs;

use regex::Regex;
use markov::Chain;

use rand::Rng;

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use chrono::{ Duration, Utc, DateTime };
use tokio::sync::{ Mutex, MutexGuard };

use async_recursion::async_recursion;

static CACHE_ENG_YML: &str = "cache/cache_eng.yml";
static CACHE_RU_YML: &str = "cache/cache_ru.yml";
static CACHE_RDN: &str = "cache/cache.rs";

// WILL NOT WORK WITH ANYTHING MORE THAN 200
// NO IDEA WHY...
static CHANNEL_CACHE_MAX: u64 = 199;

// Note: machine learning based translation is very hard without cuda
static TRANSLATION_MAX: u32 = 5;

// Note: use 66 for low activity/comfortable behavior
pub static ACTIVITY_LEVEL: AtomicU32 = AtomicU32::new(66);

lazy_static! {
  pub static ref CACHE_ENG: Mutex<Chain<String>>    = Mutex::new(Chain::new());
  pub static ref CACHE_ENG_STR: Mutex<Vec<String>>  = Mutex::new(Vec::new());
  pub static ref CACHE_RU: Mutex<Chain<String>>     = Mutex::new(Chain::new());
  pub static ref LAST_UPDATE: Mutex<DateTime<Utc>>  = Mutex::new(Utc::now());
}

pub async fn update_cache( ctx: &Context
                         , channels: &HashMap<ChannelId, GuildChannel>
                         ) {

  info!("updating AI chain has started");

  setm!{ cache_eng = CACHE_ENG.lock().await
       , cache_ru = CACHE_RU.lock().await
       , cache_eng_str = CACHE_ENG_STR.lock().await };

  if cache_eng.is_empty() || cache_ru.is_empty() {
    if fs::metadata(CACHE_ENG_YML).await.is_ok() {
      if let Ok(eng_cache) = Chain::load(CACHE_ENG_YML) {
        *cache_eng = eng_cache;
      }
    } else {
      for confuse in CONFUSION.iter() {
        cache_eng.feed_str( confuse );
      }
    }
    if fs::metadata(CACHE_RU_YML).await.is_ok() {
      if let Ok(ru_cache) = Chain::load(CACHE_RU_YML) {
        *cache_ru = ru_cache;
      }
    } else {
      for confuse in CONFUSION_RU.iter() {
        cache_ru.feed_str( confuse );
      }
    }
  }

  if cache_eng_str.is_empty() {
    if let Ok(contents) = fs::read_to_string(CACHE_RDN).await {
      if let Ok(rdn) = rudano::from_str::<Vec<String>>(&contents) {
        for res in rdn {
          cache_eng_str.push(res);
        }
      }
    } else {
      for confuse in CONFUSION.iter() {
        cache_eng_str.push( confuse.to_string() );
      }
    }
  }

  let mut ru_messages_for_translation : Vec<String> = vec![];
  set! { re1 = Regex::new(r"<(.*?)>").unwrap()
       , re2 = Regex::new(r":(.*?):").unwrap()
       , re3 = Regex::new(r"&(.*?);").unwrap() };

  let m_count = CHANNEL_CACHE_MAX * AI_LEARN.len() as u64;
  let progress_step = m_count / 5;
  let mut m_progress: u64 = 0; // progress for all channels
  let mut i_progress: u64 = 0; // progress for single channel

  for chan in channels.keys() {
    if let Some(c_name) = chan.name(&ctx).await {
      if let Some(ch_lang) = AI_LEARN.iter().find(|c| c.id == c_name) {
        let mut messages = chan.messages_iter(&ctx).boxed();

        info!("updating ai chain from {}", &c_name);
        let mut i_ru_for_translation : u32 = 0;
        let mut i: u64 = 0;

        while let Some(message_result) = messages.next().await {
          if let Ok(mmm) = message_result {
            if !mmm.author.bot && !mmm.content.starts_with('~') {
              let is_to_bot = !mmm.mentions.is_empty() && (&mmm.mentions).iter().any(|u| u.bot);
              if !is_to_bot {
                if i > CHANNEL_CACHE_MAX {
                  break;
                }
                if i_progress > progress_step {
                  let part = ((m_progress as f64/m_count as f64) * 100.0).round();
                  let status = format!("Learning {}%", part);
                  ctx.set_activity(Activity::listening(&status)).await;
                  i_progress = 0;
                } else {
                  i_progress += 1;
                }
                i += 1; m_progress += 1;
                if !check_registration(chan.0, mmm.id.0).await {
                  let mut result_string = re1.replace_all(&mmm.content, "").to_string();
                  result_string = re2.replace_all(&result_string, "").to_string();
                  result_string = re3.replace_all(&result_string, "").to_string();
                  let result = result_string.trim();
                  let is_http = result.starts_with("http");
                  if !result.is_empty() && !result.contains('$') && !is_http {
                    match ch_lang.lang {
                      ChannelLanguage::Russian => {
                        cache_ru.feed_str(result);
                        if i_ru_for_translation < TRANSLATION_MAX {
                          ru_messages_for_translation.push(result.to_string());
                          i_ru_for_translation += 1;
                        }
                      },
                      ChannelLanguage::English => {
                        cache_eng.feed_str(result);
                        if result.contains('\n') {
                          for line in result.lines() {
                            if !line.is_empty() {
                              cache_eng_str.push(line.to_string());
                            }
                          }
                        } else {
                          cache_eng_str.push(result.to_string());
                        }
                      },
                      ChannelLanguage::Bilingual => {
                        if lang::is_russian(result) {
                          cache_ru.feed_str(result);
                          if i_ru_for_translation < TRANSLATION_MAX {
                            ru_messages_for_translation.push(result.to_string());
                            i_ru_for_translation += 1;
                          }
                        } else {
                          cache_eng.feed_str(result);
                          if result.contains('\n') {
                            for line in result.lines() {
                              if !line.is_empty() {
                                cache_eng_str.push(line.to_string());
                              }
                            }
                          } else {
                            cache_eng_str.push(result.to_string());
                          }
                        }
                      }
                    }
                    register(chan.0, mmm.id.0).await;
                  }
                }
              }
            }
          }
        }
      }
    }
  }

  info!("Dumping chains!");
  let _ = cache_eng.save(CACHE_ENG_YML);
  let _ = cache_ru.save(CACHE_RU_YML);

  if let Ok(rdn) = rudano::to_string_compact(&cache_eng_str.clone()) {
    if let Err(why) = fs::write(CACHE_RDN, rdn).await {
      error!("failed save rudano cache {:?}", why);
    }
  } else {
    error!("failed to serialize cache to rudano");
  }

  if !ru_messages_for_translation.is_empty() {
    info!("Translating cache");
    tokio::spawn(async move {
      tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
      if let Ok(mut translated) = bert::ru2en_many(ru_messages_for_translation).await {
        if !translated.is_empty() {
          cache_eng_str.append(&mut translated);
          info!("Cache translation complete");
        }
      }
    });
  }

  info!("Updating cache complete");

  // enable backup/restore functionality
  RESTORE.store(true, Ordering::Relaxed);

  let version = format!("Version {}", env!("CARGO_PKG_VERSION").to_string());
  ctx.set_activity(Activity::playing(&version)).await;
}

pub async fn clear_cache() {
  setm!{ cache_eng = CACHE_ENG.lock().await
       , cache_ru = CACHE_RU.lock().await
       , cache_eng_str = CACHE_ENG_STR.lock().await };
  *cache_eng = Chain::new();
  *cache_ru = Chain::new();
  cache_eng_str.clear();
  if fs::metadata(CACHE_ENG_YML).await.is_ok() {
    let _ = fs::remove_file(CACHE_ENG_YML).await;
  }
  if fs::metadata(CACHE_RU_YML).await.is_ok() {
    let _ = fs::remove_file(CACHE_RU_YML).await;
  }
  if fs::metadata(CACHE_RDN).await.is_ok() {
    let _ = fs::remove_file(CACHE_RDN).await;
  }
  // Finally clear ZTREE
  if fs::metadata("trees/ztree.lusf").await.is_ok() {
    let _ = fs::remove_file("trees/ztree.lusf").await;
  }
}

pub async fn actualize_cache(ctx: &Context) {
  let nao = Utc::now();
  let mut last_update = LAST_UPDATE.lock().await;
  let since_last_update : Duration = nao - *last_update;
  if since_last_update > Duration::hours(2) {
    let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
    let data = ctx.data.read().await;
    if let Some(servers) = data.get::<AllGuilds>() {
      let server_ids = servers.iter()
                              .map(|srv| GuildId(srv.id))
                              .collect::<Vec<GuildId>>();
      for server in server_ids {
        if let Ok(serv_channels) = server.channels(ctx).await {
          all_channels.extend(serv_channels);
        }
      }
      update_cache(ctx, &all_channels).await;
      *last_update = nao;
    }
  }
}

pub async fn make_quote(ctx: &Context, msg: &Message, author_id: UserId) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let mut have_something = false;
  if let Some(guild_id) = msg.guild_id {
    let mut chain = Chain::new();
    let re1 = Regex::new(r"<(.*?)>").unwrap();
    let re2 = Regex::new(r":(.*?):").unwrap();
    if let Ok(channels) = guild_id.channels(&ctx).await {
      for (chan, _) in channels {
        if let Some(c_name) = chan.name(&ctx).await {
          if AI_LEARN.iter().any(|c| c.id == c_name) {
            if let Ok(messages) = chan.messages(&ctx, |r|
              r.limit(100) // 100 is max
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
      if let Ok(typing) = start_typing {
        typing.stop();
      }
      return Some(chain.generate_str());
    }
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  None
}

pub async fn generate_with_language(ctx: &Context, russian: bool) -> String {
  actualize_cache(ctx).await;
  let chain : MutexGuard<Chain<String>> =
    if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  chain.generate_str()
}

pub async fn generate(ctx: &Context, msg: &Message, mbrussian: Option<bool>) -> String {
  let msg_content = &msg.content;
  let russian = if let Some(rus) = mbrussian
    { rus } else { lang::is_russian(msg_content) };
  actualize_cache(ctx).await;
  let chain : MutexGuard<Chain<String>> =
  if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  let mut out = chain.generate_str();
  let rndx = rand::thread_rng().gen_range(0, 50);
  if rndx == 1 {
    if russian {
      out = boris::spell(&out);
    } else {
      out = uwu::spell(&out);
    }
  }
  out
}

pub fn obfuscate(msg_content: &str) -> String {
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
  let rndx = rand::thread_rng().gen_range(0, 6);
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

#[async_recursion]
async fn generate_response(ctx: &Context, msg: &Message) -> String {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
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
      } else if let Ok(answer) = bert::chat(text, msg.author.id.0).await {
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
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  if answer.as_str().trim().is_empty() {
    generate_response(ctx, msg).await
  } else {
    answer
  }
}

pub async fn chat(ctx: &Context, msg: &Message) {
  let answer = generate_response(ctx, msg).await;
  if !answer.is_empty() {
    let rnd = rand::thread_rng().gen_range(0, 3);
    if rnd == 1 {
      reply(&ctx, &msg, &answer).await;
    } else {
      channel_message(&ctx, &msg, &answer).await;
    }
  }
}

pub async fn response(ctx: &Context, msg: &Message) {
  let answer = generate_response(ctx, msg).await;
  if !answer.is_empty() {
    reply(&ctx, &msg, &answer).await;
  }
}

use crate::{
  types::common::{ AllGuilds, ChannelLanguage },
  common::{ help::lang
          , db::trees::{ register, check_registration
                       , LSUF, ZSUF }
  },
  collections::base::{ CONFUSION
                     , CONFUSION_RU },
  collections::channels::AI_LEARN,
  steins::ai::bert,
  message::RESTORE
};

use serenity::{
  prelude::*,
  model::{ id::{ GuildId, ChannelId }
         , channel::GuildChannel
         , gateway::Activity },
  futures::StreamExt
};

use std::sync::atomic::Ordering;
use async_std::fs;

use regex::Regex;
use markov::Chain;

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use chrono::{ Duration, Utc, DateTime };
use once_cell::sync::Lazy;

use kathoey::SEPARATORS;
use kathoey::types::Kathoey;
use nlprule::{Tokenizer, Rules};

static CACHE_ENG_YML: &str = "cache/cache_eng.yml";
static CACHE_RU_YML: &str = "cache/cache_ru.yml";
static CACHE_RDN: &str = "cache/cache.rs";

static NLPRULE_TOKENIZER: &str = "nlprule/en_tokenizer.bin";
static NLPRULE_RULES: &str = "nlprule/en_rules.bin";

// WILL NOT WORK WITH ANYTHING MORE THAN 200
// NO IDEA WHY...
static CHANNEL_CACHE_MAX: u64 = 199;

// Note: machine learning based translation is very hard without cuda
static TRANSLATION_MAX: u32 = 5;

// Note: use 66 for low activity/comfortable behavior
pub static ACTIVITY_LEVEL: AtomicU32 = AtomicU32::new(66);

pub static CACHE_ENG: Lazy<Mutex<Chain<String>>> =
  Lazy::new(|| Mutex::new(Chain::new()));
pub static CACHE_ENG_STR: Lazy<Mutex<Vec<String>>> =
  Lazy::new(|| Mutex::new(Vec::new()));
pub static CACHE_RU: Lazy<Mutex<Chain<String>>> =
  Lazy::new(|| Mutex::new(Chain::new()));
pub static LAST_UPDATE: Lazy<Mutex<DateTime<Utc>>> =
  Lazy::new(|| Mutex::new(Utc::now()));
pub static KATHOEY: Lazy<Mutex<Kathoey>> =
  Lazy::new(|| Mutex::new(Kathoey::load("../Kathoey/dict.bin").unwrap()));

pub static NLPR_TOKENIZER: Lazy<Mutex<Tokenizer>> =
  Lazy::new(|| Mutex::new( Tokenizer::new(NLPRULE_TOKENIZER).unwrap() ));
pub static NLPR_RULES: Lazy<Mutex<Rules>> =
  Lazy::new(|| Mutex::new( Rules::new(NLPRULE_RULES).unwrap() ));

pub static RE1: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
pub static RE2: Lazy<Regex> = Lazy::new(|| Regex::new(r":(.*?):").unwrap());
pub static RE3: Lazy<Regex> = Lazy::new(|| Regex::new(r"&(.*?);").unwrap());
pub static RE4: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub async fn reinit() {
  let mut cache_eng_str = CACHE_ENG_STR.lock().await;
  *cache_eng_str = cache_eng_str.clone()
                                .into_iter()
                                .rev()
                                .take(666)
                                .collect::<Vec<String>>();
}

pub fn process_message_for_gpt(s: &str) -> String {
  let mut result_string = RE1.replace_all(s, "").to_string();
  result_string = RE2.replace_all(&result_string, "").to_string();
  result_string = RE3.replace_all(&result_string, "").to_string();
  result_string = RE4.replace_all(&result_string, " ").to_string();
  result_string.trim().to_string()
}

pub fn process_message_string(s: &str, lang: ChannelLanguage) -> Option<(String, ChannelLanguage)> {
  let mut result_string = process_message_for_gpt(&s);
  result_string = result_string.replace(
    |c: char| !c.is_whitespace() && !c.is_alphabetic(), "");
  let result = result_string.trim();
  let is_http = result.starts_with("http");
  if !result.is_empty() && !result.contains('$') && !is_http {
    let mut result_str = result.to_string();
    let l = if lang == ChannelLanguage::Bilingual {
        if lang::is_russian(&result) {
          ChannelLanguage::Russian
        } else {
          ChannelLanguage::English
        }
      } else { lang };
    let words = result.split(&SEPARATORS[..]);
    for word in words {
      if word.starts_with("http") {
        result_str = result_str.replace(word, "");
      }
    }
    if l == ChannelLanguage::English {
      result_str = result_str.replace(
        |c: char| !c.is_whitespace() && !c.is_ascii(), "");
    }
    Some((result_str, l))
  } else {
    None
  }
}

pub async fn update_cache( ctx: &Context
                         , channels: &HashMap<ChannelId, GuildChannel>
                         ) {

  info!("updating AI chain has started");

  setm!{ cache_eng      = CACHE_ENG.lock().await
       , cache_ru       = CACHE_RU.lock().await
       , cache_eng_str  = CACHE_ENG_STR.lock().await };

  if cache_eng.is_empty() || cache_ru.is_empty() {
    if fs::metadata(CACHE_ENG_YML).await.is_ok() {
      if let Ok(eng_cache) = Chain::load(CACHE_ENG_YML) {
        *cache_eng = eng_cache;
      } else {
        error!("Failed to load chace eng YML!");
      }
    } else {
      for confuse in CONFUSION.iter() {
        cache_eng.feed_str( confuse );
      }
    }
    if fs::metadata(CACHE_RU_YML).await.is_ok() {
      if let Ok(ru_cache) = Chain::load(CACHE_RU_YML) {
        *cache_ru = ru_cache;
      } else {
        error!("Failed to load chace ru YML!");
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

  let mut ru_messages_for_translation: Vec<String> = vec![];

  let m_count = CHANNEL_CACHE_MAX * AI_LEARN.len() as u64;
  let progress_step = m_count / 5;
  let mut m_progress: u64 = 0; // progress for all channels
  let mut i_progress: u64 = 0; // progress for single channel

  for chan in channels.keys() {
    if let Some(c_name) = chan.name(&ctx).await {
      if let Some(ch_lang) = AI_LEARN.iter().find(|c| c.id == chan.0) {
        let start_typing = ctx.http.start_typing(chan.0);
        let mut messages = chan.messages_iter(&ctx).boxed();

        info!("updating ai chain from {}", &c_name);
        let mut i_ru_for_translation: u32 = 0;
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
                  debug!("#processing {}", &mmm.content);
                  if let Some((result, lang)) = process_message_string(&mmm.content, ch_lang.lang) {
                    match lang {
                      ChannelLanguage::Russian => {
                        debug!("#adding to russian {}", &result);
                        cache_ru.feed_str(&result);
                        if i_ru_for_translation < TRANSLATION_MAX {
                          ru_messages_for_translation.push(result.to_string());
                          i_ru_for_translation += 1;
                        }
                      },
                      ChannelLanguage::English => {
                        debug!("#adding to english {}", &result);
                        cache_eng.feed_str(&result);
                        if result.contains('\n') {
                          for line in result.lines() {
                            if !line.is_empty() {
                              cache_eng_str.push(line.to_string());
                            }
                          }
                        } else {
                          cache_eng_str.push(result);
                        }
                      },
                      ChannelLanguage::Bilingual => { /* we know language from process_message fn */ }
                    }
                    register(chan.0, mmm.id.0).await;
                  }
                }
              }
            }
          }
        }
        if let Ok(typing) = start_typing {
          typing.stop();
        }
      }
    }
  }

  info!("Dumping chains!");
  if let Err(err) = cache_eng.save(CACHE_ENG_YML) {
    error!("failed to save eng yml cache: {:?}", err);
  }
  if let Err(err) = cache_ru.save(CACHE_RU_YML) {
    error!("failed to save ru yml cache: {:?}", err);
  }

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
      tokio::time::sleep(std::time::Duration::from_secs(5)).await;
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

// TODO: error checks
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

  if fs::metadata(ZSUF).await.is_ok() {
    let _ = fs::remove_file(ZSUF).await;
  }
  if fs::metadata(LSUF).await.is_ok() {
    let _ = fs::remove_file(LSUF).await;
  }
}

pub async fn actualize_cache(ctx: &Context, force: bool) {
  let nao = Utc::now();
  let mut last_update = LAST_UPDATE.lock().await;
  let since_last_update: Duration = nao - *last_update;
  if since_last_update > Duration::hours(2) || force {
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

#[cfg(test)]
mod cache_tests {
  use super::*;
  #[test]
  fn cache_msg_string_process_eng_test() {
    assert_eq!(Some((String::from("Hello"), ChannelLanguage::English))
        , process_message_string("Hello", ChannelLanguage::English));
  }
  #[test]
  fn cache_msg_string_process_ru_test() {
    assert_eq!(Some((String::from("Привет"), ChannelLanguage::Russian))
        , process_message_string("Привет", ChannelLanguage::Bilingual));
    assert_eq!(Some((String::from("Бойся женщин Они мстительны и безжалостны"), ChannelLanguage::Russian))
        , process_message_string("Бойся женщин! Они мстительны и безжалостны!", ChannelLanguage::Russian));
  }
}

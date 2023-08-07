use crate::{
  salieri::SALIERI,
  types::serenity::{ AllGuilds, ChannelLanguage },
  common::{ help::lang
          , constants::PREFIX
          , db::trees::{ messages::{ register, check_registration }
                       , LSUF, ZSUF, RSUF, MSUF }
  },
  collections::base::{ CONFUSION
                     , CONFUSION_RU },
  collections::channels::AI_LEARN,
  message::RESTORE
};

use mozart::{
  cache::CACHE_ENG_STR,
  bert::process_message_for_gpt
};

use serenity::{
  prelude::*,
  model::{ id::{ GuildId, ChannelId }
         , channel::GuildChannel },
  gateway::ActivityData,
  futures::StreamExt
};

use std::sync::atomic::Ordering;
use async_std::fs;

use markov::Chain;

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use chrono::{ Duration, Utc, DateTime };
use once_cell::sync::Lazy;

use kathoey::SEPARATORS;
use kathoey::types::Kathoey;

static CACHE_ENG_YML: &str = "cache/cache_eng.yml";
static CACHE_RU_YML: &str = "cache/cache_ru.yml";
static CACHE_YML: &str = "cache/cache.yml";

// WILL NOT WORK WITH ANYTHING MORE THAN 200
// NO IDEA WHY...
static CHANNEL_CACHE_MAX: u64 = 199;

// TODO: store in some variable
pub static ACTIVITY_LEVEL: AtomicU32 = AtomicU32::new(199);

pub static CACHE_ENG: Lazy<Mutex<Chain<String>>> =
  Lazy::new(|| Mutex::new(Chain::new()));
pub static CACHE_RU: Lazy<Mutex<Chain<String>>> =
  Lazy::new(|| Mutex::new(Chain::new()));
pub static LAST_UPDATE: Lazy<Mutex<DateTime<Utc>>> =
  Lazy::new(|| Mutex::new(Utc::now()));
pub static KATHOEY: Lazy<Mutex<Kathoey>> =
  Lazy::new(|| Mutex::new(Kathoey::load("../Kathoey/dict.bin").unwrap()));

pub async fn reinit() {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    if let Err(why) = salieri.send_task(
                        mozart::cache::REINIT_CACHE::new()
                      ).await {
      error!("failed to reinit cache {why}");
    }
  }
}

pub fn process_message_string(s: &str, lang: ChannelLanguage) -> Option<(String, ChannelLanguage)> {
  let mut result_string = process_message_for_gpt(s);
  result_string = result_string.replace(
    |c: char| !c.is_whitespace() && !c.is_alphabetic(), "");
  let result = result_string.trim();
  if !result.is_empty() && !result.contains('$') {
    let mut result_str = result.to_string();
    let l = if lang == ChannelLanguage::Bilingual {
        if lang::is_russian(result) {
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
    if let Ok(contents) = fs::read_to_string(CACHE_YML).await {
      if let Ok(yml) = serde_yaml::from_str::<Vec<String>>(&contents) {
        for res in yml {
          cache_eng_str.insert(res);
        }
      }
    } else {
      for confuse in CONFUSION.iter() {
        cache_eng_str.insert( confuse.to_string() );
      }
    }
  }

  let m_count = CHANNEL_CACHE_MAX * AI_LEARN.len() as u64;
  let progress_step = m_count / 5;
  let mut m_progress: u64 = 0; // progress for all channels
  let mut i_progress: u64 = 0; // progress for single channel

  for chan in channels.keys() {
    if let Ok(c_name) = chan.name(&ctx).await {
      if let Some(ch_lang) = AI_LEARN.iter().find(|c| c.id == chan.0.get()) {
        let start_typing = ctx.http.start_typing(*chan);
        let mut messages = chan.messages_iter(&ctx).boxed();

        info!("updating ai chain from {}", &c_name);
        let mut i: u64 = 0;

        while let Some(message_result) = messages.next().await {
          if let Ok(mmm) = message_result {
            if !mmm.author.bot && !mmm.content.starts_with(PREFIX) {
              let is_to_bot = !mmm.mentions.is_empty() && mmm.mentions.iter().any(|u| u.bot);
              if !is_to_bot {
                if i > CHANNEL_CACHE_MAX {
                  break;
                }
                if i_progress > progress_step {
                  let part = ((m_progress as f64/m_count as f64) * 100.0).round();
                  let status = format!("Learning {part}%");
                  ctx.set_activity(Some( ActivityData::listening(&status) ));
                  i_progress = 0;
                } else {
                  i_progress += 1;
                }
                i += 1; m_progress += 1;
                if !check_registration(chan.0.get(), mmm.id.0.get()).await {
                  debug!("#processing {}", &mmm.content);
                  if let Some((result, lang)) = process_message_string(&mmm.content, ch_lang.lang) {
                    match lang {
                      ChannelLanguage::Russian => {
                        debug!("#adding to russian {}", &result);
                        cache_ru.feed_str(&result);
                      },
                      ChannelLanguage::English => {
                        debug!("#adding to english {}", &result);
                        cache_eng.feed_str(&result);
                        if result.contains('\n') {
                          for line in result.lines() {
                            if !line.is_empty() {
                              cache_eng_str.insert(line.to_string());
                            }
                          }
                        } else {
                          cache_eng_str.insert(result);
                        }
                      },
                      ChannelLanguage::Bilingual => { /* we know language from process_message fn */ }
                    }
                    register(chan.0.get(), mmm.id.0.get()).await;
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
    error!("failed to save eng yml cache: {err}");
  }
  if let Err(err) = cache_ru.save(CACHE_RU_YML) {
    error!("failed to save ru yml cache: {err}");
  }

  {
    let cache_str_to_save = cache_eng_str.clone();
    if let Ok(yml) = serde_yaml::to_string(&cache_str_to_save) {
      if let Err(why) = fs::write(CACHE_YML, yml).await {
        error!("failed save yaml cache str {why}");
      }
    } else {
      error!("failed to serialize cache to yaml");
    }
  }

  {
    {
      let salieri_lock = SALIERI.lock().await;
      if let Some(salieri) = &*salieri_lock {
        let cache_str_to_save = cache_eng_str.clone();
        if let Err(why) = salieri.send_task(
                            mozart::cache::SET_CACHE::new(cache_str_to_save)
                          ).await {
          error!("failed to reinit cache {why}");
        }
      }
    }
  }

  info!("Updating cache complete");

  // enable backup/restore functionality
  RESTORE.store(true, Ordering::Relaxed);

  let version = format!("Version {}", env!("CARGO_PKG_VERSION"));
  ctx.set_activity(Some( ActivityData::playing(&version) ));
}

pub async fn clear_cache() {
  setm!{ cache_eng      = CACHE_ENG.lock().await
       , cache_ru       = CACHE_RU.lock().await
       , cache_eng_str  = CACHE_ENG_STR.lock().await };
  *cache_eng  = Chain::new();
  *cache_ru   = Chain::new();
  cache_eng_str.clear();
  {
    let salieri_lock = SALIERI.lock().await;
    if let Some(salieri) = &*salieri_lock {
      let cache_str_to_save = cache_eng_str.clone();
      if let Err(why) = salieri.send_task(
                          mozart::cache::SET_CACHE::new(cache_str_to_save)
                        ).await {
        error!("failed to clear cache on salieri {why}");
      }
    }
  }
  if fs::metadata(CACHE_ENG_YML).await.is_ok() {
    let _ = fs::remove_file(CACHE_ENG_YML).await;
  }
  if fs::metadata(CACHE_RU_YML).await.is_ok() {
    let _ = fs::remove_file(CACHE_RU_YML).await;
  }
  if fs::metadata(CACHE_YML).await.is_ok() {
    let _ = fs::remove_file(CACHE_YML).await;
  }

  if fs::metadata(ZSUF).await.is_ok() {
    let _ = fs::remove_file(ZSUF).await;
  }
  if fs::metadata(LSUF).await.is_ok() {
    let _ = fs::remove_file(LSUF).await;
  }
  if fs::metadata(RSUF).await.is_ok() {
    let _ = fs::remove_file(RSUF).await;
  }
  if fs::metadata(MSUF).await.is_ok() {
    let _ = fs::remove_file(MSUF).await;
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
                              .map(|srv| GuildId( to_nzu!( srv.id ) ))
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

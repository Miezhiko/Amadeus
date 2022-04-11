use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig }
};

use std::collections::{ HashSet, HashMap };

use tch::Device;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use chrono::{ Utc, DateTime, Duration };

use celery::prelude::*;

pub static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);

pub static CACHE_ENG_STR: Lazy<Mutex<HashSet<String>>> =
  Lazy::new(|| Mutex::new(HashSet::new()));

pub fn conv_model_loader() -> ConversationModel {
  ConversationModel::new(
    ConversationConfig {
      min_length: 3,
      max_length: 64,
      min_length_for_response: 5,
      device: *DEVICE,
      ..Default::default()
    }
  ).unwrap()
}

pub static CONVMODEL: Lazy<Mutex<Option<ConversationModel>>> =
  Lazy::new(|| Mutex::new(Some(conv_model_loader())));

pub static CONVMODEL_USED: Lazy<Mutex<Option<DateTime<Utc>>>> =
  Lazy::new(|| Mutex::new(None));

#[allow(clippy::type_complexity)]
pub static CHAT_CONTEXT: Lazy<Mutex<HashMap<u64, (ConversationManager, u32)>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

#[celery::task]
pub async fn CONTEXT_CLEAR() -> TaskResult<()> {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
  Ok(())
}

#[celery::task]
pub async fn REINIT_CACHE() -> TaskResult<()> {
  let mut cache_eng_str = CACHE_ENG_STR.lock().await;
  *cache_eng_str = cache_eng_str.clone()
                                .into_iter()
                                .take(666)
                                .collect::<HashSet<String>>();
  Ok(())
}

#[celery::task]
pub async fn SET_CACHE(new_cache: HashSet<String>) -> TaskResult<()> {
  let mut cache_eng_str = CACHE_ENG_STR.lock().await;
  *cache_eng_str = new_cache;
  Ok(())
}

#[celery::task]
pub async fn MODELS_REINIT() -> TaskResult<()> {
  let mut convmodel_used = CONVMODEL_USED.lock().await;
  if let Some(conv_model_used_time) = &*convmodel_used {
    let nao = Utc::now();
    let since_last_use: Duration = nao - *conv_model_used_time;
    if since_last_use > Duration::minutes(10) {
      let mut convmodel = CONVMODEL.lock().await;
      *convmodel = None;
      *convmodel_used = None;
    }
  }
  // don't free ENRU model if CONV model is loaded.
  if convmodel_used.is_none() {
    let mut enru_used = crate::bert::translation::ENRUMODEL_USED.lock().await;
    if let Some(enru_model_used_time) = &*enru_used {
      let nao = Utc::now();
      let since_last_use: Duration = nao - *enru_model_used_time;
      if since_last_use > Duration::minutes(30) {
        let mut enrumodel = crate::bert::translation::ENRUMODEL.lock().await;
        *enrumodel = None;
        *enru_used = None;
      }
    }
  }

  Ok(())
}

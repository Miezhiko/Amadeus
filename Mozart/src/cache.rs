use crate::bert::chat;

use rust_bert::{
  resources::RemoteResource,
  pipelines::{
    conversation::{ ConversationManager
                  , ConversationModel
                  , ConversationConfig }
  }
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

const DIALOGPT_LARGE_MODEL: (&'static str, &'static str) = (
  "dialogpt-large/model",
  "https://huggingface.co/microsoft/DialoGPT-large/resolve/refs%2Fpr%2F4/rust_model.ot"
);

const DIALOGPT_LARGE_CONFIG: (&'static str, &'static str) = (
  "dialogpt-large/config",
  "https://huggingface.co/microsoft/DialoGPT-large/resolve/main/config.json",
);

const DIALOGPT_LARGE_VOCAB: (&'static str, &'static str) = (
  "dialogpt-large/vocab",
  "https://huggingface.co/microsoft/DialoGPT-large/resolve/main/vocab.json",
);

const DIALOGPT_LARGE_MERGES: (&'static str, &'static str) = (
  "dialogpt-large/merges",
  "https://huggingface.co/microsoft/DialoGPT-large/resolve/main/merges.txt",
);

pub fn conv_model_loader() -> ConversationModel {
  ConversationModel::new(
    ConversationConfig {
      model_resource: Box::new(RemoteResource::from_pretrained(
        DIALOGPT_LARGE_MODEL
      )),
      config_resource: Box::new(RemoteResource::from_pretrained(
        DIALOGPT_LARGE_CONFIG
      )),
      vocab_resource: Box::new(RemoteResource::from_pretrained(
        DIALOGPT_LARGE_VOCAB
      )),
      merges_resource: Some(Box::new(RemoteResource::from_pretrained(
        DIALOGPT_LARGE_MERGES
      ))),
      min_length: 3,
      max_length: Some(1000),
      min_length_for_response: 10,
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
  chat::reinit().await;
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
    if since_last_use > Duration::minutes(60) {
      let mut convmodel = CONVMODEL.lock().await;
      *convmodel = None;
      *convmodel_used = None;
    }
  }
  Ok(())
}

use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig }
};

use std::collections::{ HashSet, HashMap };

use tch::Device;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use chrono::{ Utc, DateTime };

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

use std::collections::HashSet;

use tch::Device;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use celery::prelude::*;

pub static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);

pub static CACHE_ENG_STR: Lazy<RwLock<HashSet<String>>> =
  Lazy::new(|| RwLock::new(HashSet::new()));

#[celery::task]
pub async fn REINIT_CACHE() -> TaskResult<()> {
  let mut cache_eng_str = CACHE_ENG_STR.write().await;
  *cache_eng_str = cache_eng_str.clone()
                                .into_iter()
                                .take(666)
                                .collect::<HashSet<String>>();
  Ok(())
}

#[celery::task]
pub async fn SET_CACHE(new_cache: HashSet<String>) -> TaskResult<()> {
  let mut cache_eng_str = CACHE_ENG_STR.write().await;
  *cache_eng_str = new_cache;
  Ok(())
}

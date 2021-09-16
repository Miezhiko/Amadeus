use crate::{
  types::w3c::PlayerAPI,
  common::{
    aka::{ self, Aka },
    constants::W3C_API
  }
};

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

pub static AKA: Lazy<Mutex<Aka>> = Lazy::new(|| Mutex::new(Aka::new()));

pub async fn check_aka( battletag: &str
                      , rqcl: &reqwest::Client ) -> Option<String> {
  let mut aka_lock = AKA.lock().await;
  match aka_lock.get(battletag) {
    Some(aka) => aka.clone(),
    None => {
      let user = battletag.replace("#","%23");
      let url = format!("{}/players/{}", W3C_API, user);
      if let Ok(res) = rqcl.get(&url).send().await {
        match res.json::<PlayerAPI>().await {
          Ok(papi) => {
            if let Some(aka) = papi.playerAkaData {
              if let Some(aka_name) = aka.name {
                aka_lock.insert(battletag.to_string(), Some(aka_name.clone()));
                if let Err(err) = aka::put_aka(&*aka_lock).await {
                  error!("failed to update aka rs db {:?}", err);
                }
                return Some(aka_name);
              } else {
                aka_lock.insert(battletag.to_string(), None);
                if let Err(err) = aka::put_aka(&*aka_lock).await {
                  error!("failed to update aka rs db {:?}", err);
                }
              }
            } else {
              aka_lock.insert(battletag.to_string(), None);
              if let Err(err) = aka::put_aka(&*aka_lock).await {
                error!("failed to update aka rs db {:?}", err);
              }
            }
          }, Err(err) => {
            warn!("Failed parse player api {:?}, url: {}", err, url);
          }
        }
      } else {
        warn!("Failed to get {}", url);
      }
      None
    }
  }
}

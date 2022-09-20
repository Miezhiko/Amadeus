use crate::{
  types::w3c::{ PlayerAPI, TeamPlayer },
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
      set!{ user  = battletag.replace('#',"%23")
          , url   = format!("{}/players/{}", W3C_API, user) };
      if let Ok(res) = rqcl.get(&url).send().await {
        match res.json::<PlayerAPI>().await {
          Ok(papi) => {
            if let Some(aka) = &papi.playerAkaData {
              if let Some(aka_name) = &aka.name {
                aka_lock.insert(battletag.to_string(), Some(aka_name.clone()));
                if let Err(err) = aka::put_aka(&aka_lock).await {
                  error!("failed to update aka rs db {:?}", err);
                }
                return Some(aka_name.to_string());
              } else {
                aka_lock.insert(battletag.to_string(), None);
                if let Err(err) = aka::put_aka(&aka_lock).await {
                  error!("failed to update aka rs db {:?}", err);
                }
              }
            } else {
              aka_lock.insert(battletag.to_string(), None);
              if let Err(err) = aka::put_aka(&aka_lock).await {
                error!("failed to update aka rs db {:?}", err);
              }
            }
          }, Err(err) => {
            warn!("Failed parse player api {:?}, url: {}", err, url);
            if let Ok(res2) = rqcl.get(&url).send().await {
              if let Ok(player_api_text) = res2.text().await {
                warn!("Text for url {}: {}", url, player_api_text);
              }
            }
          }
        }
      } else {
        warn!("Failed to get {}", url);
      }
      None
    }
  }
}

pub async fn aka( player: &TeamPlayer
                , rqcl: &reqwest::Client ) -> String {
  if let Some(aka) = check_aka(&player.battleTag, rqcl).await {
    aka
  } else {
    player.name.clone()
  }
}

use cannyls::{ lump::{LumpData, LumpId}
             , nvm::FileNvm
             , storage::Storage };

use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::{ task
           , sync::{ Mutex }
           };

use eyre::Result;

use once_cell::sync::Lazy;

pub static LSUF: &str = "trees/tree.lusf";
pub static ZSUF: &str = "trees/ztree.lusf";

#[derive(Serialize, Deserialize)]
struct Points {
  count: u64,
  streak: u64
}

fn get_storage(tree: &str) -> Storage<FileNvm> {
  if !Path::new(tree).exists() {
    let f = FileNvm::create(tree, 666666666).unwrap();
    let storage: Storage<FileNvm> = Storage::create(f).unwrap();
    storage
  } else {
    let f = FileNvm::open(tree).unwrap();
    let storage: Storage<FileNvm> = Storage::open(f).unwrap();
    storage
  }
}

pub static STORAGE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(LSUF)));
pub static ZTREE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(ZSUF)));

pub async fn register( channel_id: u64
                     , message_id: u64) {
  let mut storage = ZTREE.lock().await;
  let u64_2: u128 = (channel_id as u128) << 64 | message_id as u128; // >
  let lump_id = LumpId::new(u64_2);
  if let Ok(mbdata) = storage.get(&lump_id) {
    if mbdata.is_none() {
      if let Ok(lump_data) = LumpData::new(vec![1]) {
        match storage.put(&lump_id, &lump_data) {
          Ok(added) => {
            if !added {
              error!("error on msg registration");
            }
          }, Err(not_added) => {
            error!("error on msg registration {:?}", not_added);
          }
        }
        if let Err(khm) = storage.journal_sync() {
          error!("failed to sync {:?}", khm);
        }
      }
    }
  }
}

pub async fn check_registration(channel_id: u64, message_id: u64) -> bool {
  let mut storage = ZTREE.lock().await;
  let u64_2: u128 = (channel_id as u128) << 64 | message_id as u128; // >
  if let Ok(mbdata) = storage.get(&LumpId::new(u64_2)) {
    mbdata.is_some()
  } else {
    false
  }
}

pub async fn add_points( guild_id: u64
                       , user_id: u64
                       , new_points: u64) {
  let mut storage = STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
        if let Some(mut data) = mbdata {
          let byte_data: &mut [u8] = data.as_bytes_mut();
          if let Ok(mut points) = bincode::deserialize::<Points>(byte_data) {
            points.count += new_points;
            if let Ok(new_bytes) = bincode::serialize(&points) {
              (*byte_data).copy_from_slice(&new_bytes[..]);
              match storage.put(&lump_id, &data) {
                Ok(added) => {
                  if added {
                    error!("error updating points");
                  }
                }, Err(not_added) => {
                  error!("failed to add points {:?}", not_added);
                }
              }
            }
          }
        } else {
          let points = Points { count: 0, streak: 0 };
          if let Ok(encoded) = bincode::serialize(&points) {
            if let Ok(lump_data) = LumpData::new(encoded) {
              match storage.put(&lump_id, &lump_data) {
                Ok(added) => {
                  if !added {
                    error!("error on points initialization");
                  }
                }, Err(not_added) => {
                  error!("error on points initialization {:?}", not_added);
                }
              }
            }
          }
        }
        if let Err(khm) = storage.journal_sync() {
          error!("failed to sync {:?}", khm);
        }
      }, Err(why) => {
        error!("Failed to get key: {:?}", why);
      }
    }
  }).await.unwrap();
}

pub async fn give_points( guild_id: u64
                        , user_id: u64
                        , target_user_id: u64
                        , points_count: u64) -> (bool, String) {
  let mut storage = STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let tu64_2: u128 = (guild_id as u128) << 64 | target_user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  let target_lump_id: LumpId = LumpId::new(tu64_2);
  match storage.get(&lump_id) {
    Ok(mbdata) => {
      if let Some(mut data) = mbdata {
        let byte_data: &mut [u8] = data.as_bytes_mut();
        let mut points: Points = bincode::deserialize(byte_data).unwrap();
        if points.count >= points_count {
          points.count -= points_count;
          let new_bytes = bincode::serialize(&points).unwrap();
          (*byte_data).copy_from_slice(&new_bytes[..]);
          let added: bool = storage.put(&lump_id, &data).unwrap();
          if added {
            error!("Some strange error updating giver points");
          }
          match storage.get(&target_lump_id) {
            Ok(tmbdata) => {
              if let Some(mut tdata) = tmbdata {
                let tbyte_data: &mut [u8] = tdata.as_bytes_mut();
                let mut tpoints: Points = bincode::deserialize(tbyte_data).unwrap();
                tpoints.count += points_count;
                let tnew_bytes = bincode::serialize(&tpoints).unwrap();
                (*tbyte_data).copy_from_slice(&tnew_bytes[..]);
                let tadded: bool = storage.put(&target_lump_id, &tdata).unwrap();
                if tadded {
                  error!("Some strange error updating receiver points");
                }
              } else {
                let tpoints = Points { count: points_count, streak: 0 };
                let tencoded: Vec<u8> = bincode::serialize(&tpoints).unwrap();
                let tlump_data: LumpData = LumpData::new(tencoded).unwrap();
                let tadded: bool = storage.put(&target_lump_id, &tlump_data).unwrap();
                if !tadded {
                  error!("Some strange error updating receiver points");
                }
              }
              if let Err(khm) = storage.journal_sync() {
                error!("failed to sync {:?}", khm);
              }
              (true, format!("{} points transfered", points_count))
            }, Err(why) => {
              error!("Failed to get key: {:?}", why);
              (false, String::from("error accessing points"))
            }
          }
        } else {
          (false, String::from("not enough points to give"))
        }
      } else {
        (false, String::from("you have no points to give"))
      }
    }, Err(why) => {
      error!("Failed to get key: {:?}", why);
      (false, String::from("error accessing points"))
    }
  }
}

pub async fn get_points(guild_id: u64, user_id: u64) -> eyre::Result<u64> {
  let mut storage = STORAGE.lock().await;
  task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    if let Ok(mbdata) = storage.get(&lump_id) {
      if let Some(mut data) = mbdata {
        let byte_data: &mut [u8] = data.as_bytes_mut();
        match bincode::deserialize::<Points>(byte_data) {
          Ok(points) => return Ok(points.count),
          Err(get_error) => {
            error!("Get error: {:?}", get_error);
            return Ok(0);
          }
        };
      }
    }
    Ok(0)
  }).await?
}

pub async fn clear_points(guild_id: u64, user_id: u64) -> Result<bool, cannyls::Error> {
  let mut storage = STORAGE.lock().await;
  task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    storage.delete(&lump_id)
  }).await.unwrap()
}

pub async fn add_win_points( guild_id: u64
                           , user_id: u64 ) -> u64 {
  let mut storage = STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
        if let Some(mut data) = mbdata {
          let byte_data: &mut [u8] = data.as_bytes_mut();
          if let Ok(mut points) = bincode::deserialize::<Points>(byte_data) {
            points.count += 10;
            points.streak += 1;
            if points.streak > 3 {
              let points_multiplier = points.streak - 3;
              points.count += 5 * points_multiplier;
            }
            if let Ok(new_bytes) = bincode::serialize(&points) {
              (*byte_data).copy_from_slice(&new_bytes[..]);
              match storage.put(&lump_id, &data) {
                Ok(added) => {
                  if added {
                    error!("error updating points");
                  }
                  if let Err(khm) = storage.journal_sync() {
                    error!("failed to sync {:?}", khm);
                  }
                  points.streak
                }, Err(ecn) => {
                  error!("Something wrong with cannyls: {:?}", ecn);
                  0
                }
              }
            } else { 0 }
          } else { 0 }
        } else {
          let points = Points { count: 10, streak: 1 };
          if let Ok(encoded) = bincode::serialize(&points) {
            if let Ok(lump_data) = LumpData::new(encoded) {
              if let Ok(added) = storage.put(&lump_id, &lump_data) {
                if !added {
                  error!("error on points initialization");
                }
                if let Err(khm) = storage.journal_sync() {
                  error!("failed to sync {:?}", khm);
                }
                1
              } else {
                error!("Something is wrong with cannyls");
                0
              }
            } else { 0 }
          } else { 0 }
        }
      }, Err(why) => {
        error!("Failed to get key: {:?}", why);
        0
      }
    }
  }).await.unwrap()
}

pub async fn break_streak( guild_id: u64
                         , user_id: u64 ) {
  let mut storage = STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
      if let Some(mut data) = mbdata {
          let byte_data: &mut [u8] = data.as_bytes_mut();
          if let Ok(mut points) = bincode::deserialize::<Points>(byte_data) {
            points.streak = 0;
            if let Ok(new_bytes) = bincode::serialize(&points) {
              (*byte_data).copy_from_slice(&new_bytes[..]);
              if let Ok(added) = storage.put(&lump_id, &data) {
                if added {
                  error!("error updating points");
                }
                if let Err(khm) = storage.journal_sync() {
                  error!("failed to sync {:?}", khm);
                }
              }
            }
          }
        }
      }, Err(why) => {
        error!("Failed to get key: {:?}", why);
      }
    }
  }).await.unwrap();
}

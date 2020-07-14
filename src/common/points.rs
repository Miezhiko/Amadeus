use cannyls::lump::{LumpData, LumpId};
use cannyls::nvm::FileNvm;
use cannyls::storage::Storage;

use std::path::Path;

use serde::{Deserialize, Serialize};

use tokio::task;
use tokio::sync::{ Mutex };

#[derive(Serialize, Deserialize)]
struct Points {
  count: u64,
  role: u64
}

fn get_storage() -> Storage<FileNvm> {
  let db_name = "tree.lusf";
  if !Path::new(db_name).exists() {
    let f = FileNvm::create(db_name, 666_666_666).unwrap();
    let storage: Storage<FileNvm> = Storage::create(f).unwrap();
    storage
  } else {
    let f = FileNvm::open(db_name).unwrap();
    let storage: Storage<FileNvm> = Storage::open(f).unwrap();
    storage
  }
}

lazy_static! {
  pub static ref STORAGE: Mutex<Storage<FileNvm>> = Mutex::new(get_storage());
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
          let mut points : Points = bincode::deserialize(byte_data).unwrap();
          points.count += new_points;
          let new_bytes = bincode::serialize(&points).unwrap();
          (*byte_data).copy_from_slice(&new_bytes[..]);
          let added: bool = storage.put(&lump_id, &data).unwrap();
          if added {
            error!("error updating points");
          }
        } else {
          let points = Points { count: 0, role: 0 };
          let encoded: Vec<u8> = bincode::serialize(&points).unwrap();
          let lump_data: LumpData = LumpData::new(encoded).unwrap();
          let added: bool = storage.put(&lump_id, &lump_data).unwrap();
          if !added {
            error!("error on points initialization");
          }
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
        let mut points : Points = bincode::deserialize(byte_data).unwrap();
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
                let mut tpoints : Points = bincode::deserialize(tbyte_data).unwrap();
                tpoints.count += points_count;
                let tnew_bytes = bincode::serialize(&tpoints).unwrap();
                (*tbyte_data).copy_from_slice(&tnew_bytes[..]);
                let tadded: bool = storage.put(&target_lump_id, &tdata).unwrap();
                if tadded {
                  error!("Some strange error updating receiver points");
                }
              } else {
                let tpoints = Points { count: points_count, role: 0 };
                let tencoded: Vec<u8> = bincode::serialize(&tpoints).unwrap();
                let tlump_data: LumpData = LumpData::new(tencoded).unwrap();
                let tadded: bool = storage.put(&target_lump_id, &tlump_data).unwrap();
                if !tadded {
                  error!("Some strange error updating receiver points");
                }
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

pub async fn get_points(guild_id: u64, user_id: u64) -> Result<u64, cannyls::Error> {
  let mut storage = STORAGE.lock().await;
  task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    if let Ok(mbdata) = storage.get(&lump_id) {
      if let Some(mut data) = mbdata {
        let byte_data: &mut [u8] = data.as_bytes_mut();
        let points : Points = bincode::deserialize(byte_data).unwrap();
        return Ok(points.count);
      }
    }
    Ok(0)
  }).await.unwrap()
}

pub async fn clear_points(guild_id: u64, user_id: u64) -> Result<bool, cannyls::Error> {
  let mut storage = STORAGE.lock().await;
  task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    storage.delete(&lump_id)
  }).await.unwrap()
}

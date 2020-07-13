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

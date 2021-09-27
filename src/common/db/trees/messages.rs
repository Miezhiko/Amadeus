use crate::common::db::trees;

use cannyls::lump::{ LumpData, LumpId };

pub async fn register( channel_id: u64
                     , message_id: u64) {
  let mut storage = trees::ZTREE.lock().await;
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
  let mut storage = trees::ZTREE.lock().await;
  let u64_2: u128 = (channel_id as u128) << 64 | message_id as u128; // >
  if let Ok(mbdata) = storage.get(&LumpId::new(u64_2)) {
    mbdata.is_some()
  } else {
    false
  }
}

use crate::common::db::trees;

use cannyls::lump::{ LumpData, LumpId };

use tokio::task;

pub async fn update_roles( guild_id: &u64
                         , user_id: &u64
                         , roles: &[u64] ) {
  let mut storage = trees::RTREE.lock().await;
  let u64_2: u128 = (*guild_id as u128) << 64 | *user_id as u128; // >
  let lump_id = LumpId::new(u64_2);
  if let Ok(mbdata) = storage.get(&lump_id) {
    if mbdata.is_some() {
      let _ = storage.delete(&lump_id);
      if let Err(khm) = storage.journal_sync() {
        error!("failed to sync {khm}");
      }
    }
    if let Ok(encoded) = bincode::serialize(roles) {
      if let Ok(lump_data) = LumpData::new(encoded) {
        match storage.put(&lump_id, &lump_data) {
          Ok(not_added) => {
            if !not_added {
              error!("error on msg registration");
            }
          }, Err(not_added) => {
            error!("error on msg registration {not_added}");
          }
        }
        if let Err(khm) = storage.journal_sync() {
          error!("failed to sync {khm}");
        }
      }
    }
  }
}

pub async fn restore_roles(guild_id: &u64, user_id: &u64) -> anyhow::Result<Vec<u64>> {
  let mut storage = trees::RTREE.lock().await;
  let u64_2: u128 = (*guild_id as u128) << 64 | *user_id as u128; // >
  task::spawn_blocking(move || {
    let lump_id: LumpId = LumpId::new(u64_2);
    if let Ok(Some(mut data)) = storage.get(&lump_id) {
      let byte_data: &mut [u8] = data.as_bytes_mut();
      match bincode::deserialize::<Vec<u64>>(byte_data) {
        Ok(roles) => return Ok(roles),
        Err(error) => {
          error!("Error trying to restore roles: {error}");
          return Ok(vec![]);
        }
      };
    }
    Ok(vec![])
  }).await?
}

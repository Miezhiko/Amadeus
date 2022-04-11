use crate::common::db::trees;

use std::collections::HashMap;

use cannyls::lump::{ LumpData, LumpId };

use bincode::config;

pub async fn register_message( guild_id: &u64
                             , message_id: &u64
                             , emoji_id: &u64
                             , role_id: &u64 ) {
  let mut storage = trees::MTREE.lock().await;
  let u64_2: u128 = (*guild_id as u128) << 64 | *message_id as u128; // >
  let lump_id = LumpId::new(u64_2);
  if let Ok(mbdata) = storage.get(&lump_id) {
    if let Some(data) = mbdata {
      let byte_data: &[u8] = data.as_bytes();
      if let Ok((emoji_roles, _len)) = bincode::decode_from_slice( byte_data
                                                                 , config::standard() ) {
        let mut emoji_role: HashMap<u64, u64> = emoji_roles;
        emoji_role.insert(*emoji_id, *role_id);
        { // delte existing node
          let _ = storage.delete(&lump_id);
          if let Err(khm) = storage.journal_sync() {
            error!("failed to sync {khm}");
          }
        }
        if let Ok(encoded) = bincode::encode_to_vec(&emoji_role, config::standard()) {
          if let Ok(lump_data) = LumpData::new(encoded) {
            match storage.put(&lump_id, &lump_data) {
              Ok(not_added) => {
                if !not_added {
                  error!("error updating message emoji roles");
                }
                if let Err(khm) = storage.journal_sync() {
                  error!("failed to sync {khm}");
                }
              }, Err(ecn) => {
                error!("Something wrong with cannyls: {ecn}");
              }
            }
          }
        }
      }
    } else {
      let mut emoji_role: HashMap<u64, u64> = HashMap::new();
      emoji_role.insert(*emoji_id, *role_id);
      if let Ok(encoded) = bincode::encode_to_vec(&emoji_role, config::standard()) {
        if let Ok(lump_data) = LumpData::new(encoded) {
          match storage.put(&lump_id, &lump_data) {
            Ok(added) => {
              if !added {
                error!("error on msg registration");
              }
            }, Err(not_added) => {
              error!("error on msg registration {not_added}");
            }
          }
          if let Err(khm) = storage.journal_sync() {
            error!("failed to sync {khm}");
          }
        } else {
          error!("failed to convert data to lump");
        }
      }
    }
  }
}

pub async fn message_roles( guild_id: &u64
                          , message_id: &u64
                          ) -> anyhow::Result<Option<HashMap<u64, u64>>> {
  let mut storage = trees::MTREE.lock().await;
  let u64_2: u128 = (*guild_id as u128) << 64 | *message_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  if let Ok(Some(mut data)) = storage.get(&lump_id) {
    let byte_data: &mut [u8] = data.as_bytes_mut();
    match bincode::decode_from_slice( byte_data, config::standard() ) {
      Ok((roles, _len)) => Ok(Some(roles)),
      Err(error) => {
        error!("Error trying to get message roles roles: {error}");
        Ok(None)
      }
    }
  } else {
    warn!("found no node in database for current message");
    Ok(None)
  }
}

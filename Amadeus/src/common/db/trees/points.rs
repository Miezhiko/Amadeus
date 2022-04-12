use crate::common::db::trees;

use cannyls::lump::{ LumpData, LumpId };

use tokio::task;

use mozart::prelude::BINCODE_CONFIG;

#[derive(bincode::Encode, bincode::Decode)]
struct Points {
  count: u64,
  streak: u64
}

pub async fn add_points( guild_id: u64
                       , user_id: u64
                       , new_points: u64) {
  let mut storage = trees::STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  if let Err(awhy) = task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
        if let Some(data) = mbdata {
          let byte_data: &[u8] = data.as_bytes();
          if let Ok((mut points, _len)) = bincode::decode_from_slice::<Points,_>(byte_data, BINCODE_CONFIG) {
            points.count += new_points;
            if let Ok(new_bytes) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
              if let Ok(lump_data_new) = LumpData::new(new_bytes) {
                match storage.put(&lump_id, &lump_data_new) {
                  Ok(added) => {
                    if added {
                      error!("error updating points");
                    }
                  }, Err(not_added) => {
                    error!("failed to add points {not_added}");
                  }
                }
              } else {
                error!("LumpData error");
              }
            }
          }
        } else {
          let points = Points { count: 0, streak: 0 };
          if let Ok(encoded) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
            if let Ok(lump_data) = LumpData::new(encoded) {
              match storage.put(&lump_id, &lump_data) {
                Ok(added) => {
                  if !added {
                    error!("error on points initialization");
                  }
                }, Err(not_added) => {
                  error!("error on points initialization {not_added}");
                }
              }
            }
          }
        }
        if let Err(khm) = storage.journal_sync() {
          error!("failed to sync {khm}");
        }
      }, Err(why) => {
        error!("Failed to get key: {why}");
      }
    }
  }).await {
    error!("failed to spawn tokio task {awhy}")
  }
}

pub async fn give_points( guild_id: u64
                        , user_id: u64
                        , target_user_id: u64
                        , points_count: u64) -> (bool, String) {
  let mut storage = trees::STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let tu64_2: u128 = (guild_id as u128) << 64 | target_user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  let target_lump_id: LumpId = LumpId::new(tu64_2);
  match storage.get(&lump_id) {
    Ok(mbdata) => {
      if let Some(mut data) = mbdata {
        let byte_data: &mut [u8] = data.as_bytes_mut();
        if let Ok((mut points, _len)) = bincode::decode_from_slice::<Points,_>(byte_data, BINCODE_CONFIG) {
          if points.count >= points_count {
            points.count -= points_count;
            if let Ok(new_bytes) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
              if let Ok(lump_data_new) = LumpData::new(new_bytes) {
                if let Ok(added) = storage.put(&lump_id, &lump_data_new) {
                  if added {
                    error!("Some strange error updating giver points");
                  }
                }
              } else {
                error!("LumpData Error");
              }
            } else {
              error!("Error encoding ponts");
            }
            match storage.get(&target_lump_id) {
              Ok(tmbdata) => {
                if let Some(mut tdata) = tmbdata {
                  let tbyte_data: &mut [u8] = tdata.as_bytes_mut();
                  if let Ok((mut tpoints, _len)) = bincode::decode_from_slice::<Points,_>(tbyte_data, BINCODE_CONFIG) {
                    tpoints.count += points_count;
                    if let Ok(tnew_bytes) = bincode::encode_to_vec(&tpoints, BINCODE_CONFIG) {
                      if let Ok(lump_data_new) = LumpData::new(tnew_bytes) {
                        if let Ok(tadded) = storage.put(&target_lump_id, &lump_data_new) {
                          if tadded {
                            error!("Some strange error updating receiver points");
                          }
                        }
                      } else {
                        error!("LumpData Error");
                      }
                    } else {
                      error!("Error encoding ponts");
                    }
                  }
                } else {
                  let tpoints = Points { count: points_count, streak: 0 };
                  if let Ok(tencoded) = bincode::encode_to_vec(&tpoints, BINCODE_CONFIG) {
                    if let Ok(tlump_data) = LumpData::new(tencoded) {
                      if let Ok(tadded) = storage.put(&target_lump_id, &tlump_data) {
                        if !tadded {
                          error!("Some strange error updating receiver points");
                        }
                      }
                    }
                  } else {
                    error!("Error encoding ponts");
                  }
                }
                if let Err(khm) = storage.journal_sync() {
                  error!("failed to sync {:?}", khm);
                }
                (true, format!("{points_count} points transfered"))
              }, Err(why) => {
                error!("Failed to get key: {why}");
                (false, String::from("error accessing points"))
              }
            }
          } else {
            (false, String::from("not enough points to give"))
          }
        } else {
          (false, String::from("you have no points to give"))
        }
      } else {
        (false, String::from("error decoding points"))
      }
    }, Err(why) => {
      error!("Failed to get key: {why}");
      (false, String::from("error accessing points"))
    }
  }
}

pub async fn get_points(guild_id: u64, user_id: u64) -> anyhow::Result<u64> {
  let mut storage = trees::STORAGE.lock().await;
  task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    if let Ok(Some(mut data)) = storage.get(&lump_id) {
      let byte_data: &mut [u8] = data.as_bytes_mut();
      match bincode::decode_from_slice::<Points,_>(byte_data, BINCODE_CONFIG) {
        Ok((points, _len)) => return Ok(points.count),
        Err(get_error) => {
          error!("Get error: {get_error}");
          return Ok(0);
        }
      };
    }
    Ok(0)
  }).await?
}

pub async fn clear_points(guild_id: u64, user_id: u64) -> Result<bool, cannyls::Error> {
  let mut storage = trees::STORAGE.lock().await;
  match task::spawn_blocking(move || {
    let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
    let lump_id: LumpId = LumpId::new(u64_2);
    storage.delete(&lump_id)
  }).await {
    Ok(r) => r,
    Err(why) => {
      error!("error clearing points {why}");
      Ok(false)
    }
  }
}

pub async fn add_win_points( guild_id: u64
                           , user_id: u64 ) -> u64 {
  let mut storage = trees::STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  match task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
        if let Some(data) = mbdata {
          let byte_data: &[u8] = data.as_bytes();
          if let Ok((mut points, _len)) = bincode::decode_from_slice::<Points,_>(byte_data, BINCODE_CONFIG) {
            points.count += 10;
            points.streak += 1;
            if points.streak > 3 {
              let points_multiplier = points.streak - 3;
              points.count += 5 * points_multiplier;
            }
            if let Ok(new_bytes) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
              if let Ok(lump_data_tnew) = LumpData::new(new_bytes) {
                match storage.put(&lump_id, &lump_data_tnew) {
                  Ok(added) => {
                    if added {
                      error!("error updating points");
                    }
                    if let Err(khm) = storage.journal_sync() {
                      error!("failed to sync {khm}");
                    }
                    points.streak
                  }, Err(ecn) => {
                    error!("Something wrong with cannyls: {ecn}");
                    0
                  }
                }
              } else {
                error!("Error putting new ponts to LumpData"); 0
              }
            } else {
              error!("Error encoding ponts"); 0
            }
          } else { 0 }
        } else {
          let points = Points { count: 10, streak: 1 };
          if let Ok(encoded) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
            if let Ok(lump_data) = LumpData::new(encoded) {
              if let Ok(added) = storage.put(&lump_id, &lump_data) {
                if !added {
                  error!("error on points initialization");
                }
                if let Err(khm) = storage.journal_sync() {
                  error!("failed to sync {khm}");
                }
                1
              } else {
                error!("Something is wrong with cannyls");
                0
              }
            } else { 0 }
          } else {
            error!("Error encoding ponts"); 0
          }
        }
      }, Err(why) => {
        error!("Failed to get key: {why}");
        0
      }
    }
  }).await {
    Ok(r) => r,
    Err(why) => {
      error!("failed to add win {why}"); 0
    }
  }
}

pub async fn break_streak( guild_id: u64
                         , user_id: u64 ) {
  let mut storage = trees::STORAGE.lock().await;
  let u64_2: u128 = (guild_id as u128) << 64 | user_id as u128; // >
  let lump_id: LumpId = LumpId::new(u64_2);
  if let Err(why) = task::spawn_blocking(move || {
    match storage.get(&lump_id) {
      Ok(mbdata) => {
        if let Some(data) = mbdata {
          let byte_data: &[u8] = data.as_bytes();
          if let Ok((mut points, _len)) = bincode::decode_from_slice::<Points,_>(byte_data, BINCODE_CONFIG) {
            points.streak = 0;
            if let Ok(new_bytes) = bincode::encode_to_vec(&points, BINCODE_CONFIG) {
              if let Ok(lump_data_new) = LumpData::new(new_bytes) {
                if let Ok(added) = storage.put(&lump_id, &lump_data_new) {
                  if added {
                    error!("error updating points");
                  }
                  if let Err(khm) = storage.journal_sync() {
                    error!("failed to sync {khm}");
                  }
                }
              } else {
                error!("LumpData Error")
              }
            } else {
              error!("Error encoding ponts");
            }
          }
        }
      }, Err(why) => {
        error!("Failed to get key: {why}");
      }
    }
  }).await {
    error!("{why}");
  }
}

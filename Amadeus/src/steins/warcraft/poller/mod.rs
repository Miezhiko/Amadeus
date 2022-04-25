pub mod finished;
pub mod bet_fields;
pub mod checker;

use crate::types::tracking::TrackingGame;

use std::collections::HashMap;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

pub static GAMES: Lazy<Mutex<HashMap<String, TrackingGame>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

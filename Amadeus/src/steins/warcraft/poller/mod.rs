pub mod finished;
pub mod bet_fields;
pub mod checker;

use crate::types::tracking::TrackingGame;

use std::collections::HashMap;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

pub static GAMES: Lazy<RwLock<HashMap<String, TrackingGame>>>
  = Lazy::new(|| RwLock::new(HashMap::new()));

#[cfg(feature = "trackers")] mod tracking;
#[cfg(feature = "trackers")] pub mod behavior;

use std::sync::atomic::AtomicU64;

use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use chrono::{ Utc, DateTime };

pub static LAST_CHANNEL: AtomicU64 = AtomicU64::new(0);
pub static START_TIME: Lazy<RwLock<DateTime<Utc>>> =
  Lazy::new(|| RwLock::new(Utc::now()));

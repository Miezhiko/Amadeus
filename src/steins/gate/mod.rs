mod social;
mod tracking;
pub mod behavior;

use std::{
  sync::atomic::AtomicU64
};

pub static LAST_CHANNEL: AtomicU64 = AtomicU64::new(0);

#![type_length_limit="2792949"]

// stable on 1.76+ but not still stable on older
#![allow(stable_features)]
#![feature(arc_unwrap_or_clone)]

#![feature(const_option)]

#[macro_use] extern crate serde;
#[macro_use] extern crate anyhow;
#[macro_use] extern crate tracing;
#[macro_use] mod macros;

mod types;

#[macro_use] mod common;

mod collections; mod commands; mod steins;

#[cfg(feature = "spam_filter")]
mod spam;

pub mod salieri;

#[cfg(feature = "naoko")]
mod naoko;

mod message; mod handler; mod checks;
mod slash; mod hooks; mod groups; mod amadeus;

#[cfg(all(feature = "salieri", feature = "naoko"))]
compile_error!("Feature salieri and naoko are mutually exclusive and cannot be enabled together");

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  let iopts = common::options::get_ioptions()
                .map_err(|e| anyhow!("Failed to parse Dhall config {e}"))?;
  if let Err(err) = amadeus::run(iopts).await {
    panic!("Amadeus died {err}")
  }
  Ok(())
}

#[cfg(test)]
mod main_tests {
  use super::*;
  #[ignore]
  #[test]
  fn conf_dhall() -> Result<(), String> {
    if let Err(why) = common::options::get_ioptions() {
      Err(format!("Bad config {why}"))
    } else {
      Ok(())
    }
  }
}

#![type_length_limit="2792949"]

#![feature(stmt_expr_attributes)]

#[macro_use] extern crate serde;
#[macro_use] extern crate anyhow;
#[macro_use] extern crate tracing;
#[macro_use] mod macros;

mod types;
mod common;
mod collections;
mod commands;
mod steins;
mod message;
mod handler;
mod checks;
mod slash;
mod hooks;
mod groups;
mod amadeus;

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  let iopts = common::options::get_ioptions()
                .map_err(|e| anyhow!("Failed to parse Dhall config {:?}", e))?;
  if let Err(err) = amadeus::run(&iopts).await {
    panic!("Amadeus died {:?}", err)
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
      Err(format!("Bad config {:?}", why))
    } else {
      Ok(())
    }
  }
}

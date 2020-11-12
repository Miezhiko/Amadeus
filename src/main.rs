#![type_length_limit="2792949"]

#[macro_use] extern crate serde;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate eyre;
#[macro_use] extern crate tracing;
#[macro_use] mod macros;

mod types;
mod common;
mod collections;
mod commands;
mod steins;
mod handler;
mod amadeus;

use eyre::{ WrapErr, Result };

#[tokio::main(core_threads=8)]
async fn main() -> Result<()> {
  let iopts = common::options::get_ioptions()
                .wrap_err("Failed to parse Dhall condig")?;
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

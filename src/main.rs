#[macro_use] extern crate serde;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] mod macros;

mod types;
mod common;
mod collections;
mod commands;
mod steins;
mod handler;
mod amadeus;

use jane_eyre::{ eyre::WrapErr, Result };

#[tokio::main(core_threads=8)]
async fn main() -> Result<()> {
  jane_eyre::install()?;
  let iopts = common::options::get_ioptions()
                .wrap_err("Failed to parse Dhall condig")?;
  if let Err(err) = amadeus::run(&iopts).await {
    panic!("Amadeus died {:?}", err)
  }
  Ok(())
}

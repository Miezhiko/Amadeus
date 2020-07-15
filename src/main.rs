#[macro_use] extern crate serde;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

#[macro_use] mod common;
mod collections;
mod commands;
mod stains;
mod handler;
mod amadeus;

#[tokio::main(core_threads=8)]
async fn main() {
  let conf = common::conf::parse_config();
  if let Err(err) = amadeus::run(&conf).await {
    panic!("Amadeus died {:?}", err)
  }
}

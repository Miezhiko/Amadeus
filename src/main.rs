#[macro_use] extern crate serde;
extern crate chrono;
extern crate typemap;
extern crate argparse;
extern crate ini;
extern crate env_logger;
extern crate rand;
extern crate regex;
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;
extern crate ical;
extern crate reqwest;
extern crate markov;
extern crate ucd;
extern crate comfy_table;
extern crate futures_util;
extern crate qrcode;
extern crate cannyls;
extern crate bincode;

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate serenity;

#[macro_use] pub mod common;
pub mod collections;
pub mod commands;
pub mod stains;

mod handler;
mod amadeus;

#[tokio::main(core_threads=8)]
async fn main() {
  let conf = common::conf::parse_config();
  if let Err(err) = amadeus::run(&conf).await {
    panic!("Amadeus died {:?}", err)
  }
}

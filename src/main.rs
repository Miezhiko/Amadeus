#![feature(proc_macro_hygiene)]

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

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate serenity;

#[macro_use] pub mod common;
pub mod conf;
pub mod collections;
pub mod commands;
pub mod stains;

mod handler;
mod amadeus;

fn main() {
  let mut conf = conf::parse_config();
  if let Err(err) = amadeus::run(&mut conf) {
    panic!("Amadeus died {:?}", err)
  }
}

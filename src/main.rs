extern crate chrono;
extern crate typemap;
extern crate argparse;
extern crate ini;
extern crate env_logger;
extern crate rand;
extern crate regex;
extern crate serde_json;
extern crate ssh2;
extern crate uuid;

#[macro_use] extern crate log;
extern crate serenity;

#[macro_use] pub mod macros;

extern crate curl;

pub mod common;
pub mod conf;
pub mod types;
pub mod collections;
pub mod commands;

mod handler;
mod amadeus;

fn main() {
  let mut conf = conf::parse_config();
  if let Err(err) = amadeus::run(&mut conf) {
    panic!("Amadeus died {:?}", err)
  }
}

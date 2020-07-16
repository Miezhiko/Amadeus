#[macro_use] extern crate serde;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] mod macros;

mod types;
mod common;
mod collections;
mod commands;
mod stains;
mod handler;
mod amadeus;

#[tokio::main(core_threads=8)]
async fn main() {
  match common::options::get_ioptions() {
    Ok(iopts) =>
      if let Err(err) = amadeus::run(&iopts).await {
        panic!("Amadeus died {:?}", err)
      },
    Err(why) => panic!("Failed to parse dhall {:?}", why)
  }
}

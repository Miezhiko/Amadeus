#[macro_use] extern crate log;
#[macro_use] extern crate anyhow;
extern crate serde;

pub mod types;
pub mod cache;
pub mod bert;
pub mod chat;

pub mod commands;
pub mod prelude;

use celery::{ Celery, self, prelude::* };

use std::sync::Arc;

pub const SALIERI_SERVICE: &str = "salieri";

// 5672 is default AMPQ port
pub const SALIERI_AMPQ: &str = "amqp://127.0.0.1:5672/%2f";

#[celery::task]
pub async fn AMADEUS_INIT() -> TaskResult<()> {
  // init
  Ok(())
}

pub async fn celery_init(ampq: &str) -> Result<Arc<Celery>, CeleryError> {
  celery::app!(
    broker = AMQPBroker { String::from( ampq ) },
    tasks = [ AMADEUS_INIT
            , cache::REINIT_CACHE
            , cache::SET_CACHE
            , chat::CHAT
            ],
    task_routes = [
      "*" => SALIERI_SERVICE,
    ],
    heartbeat = Some( 600 ),
    broker_connection_timeout = 300
  ).await
}

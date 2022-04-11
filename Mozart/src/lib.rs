#[macro_use] extern crate log;
#[macro_use] extern crate anyhow;
extern crate serde;

pub mod types;
pub mod cache;
pub mod bert;
pub mod commands;
pub mod prelude;

use celery::{ Celery, self, prelude::* };

use std::sync::Arc;

pub const SALIERI_SERVICE: &str = "salieri";
pub const AMADEUS: &str = "amadeus";

// 5672 is default AMPQ port
pub const SALIERI_AMPQ: &str = "amqp://localhost:5672//";

#[celery::task]
pub async fn AMADEUS_INIT() -> TaskResult<()> {
  // init
  Ok(())
}

pub async fn celery_init(ampq: &str) -> Result<Arc<Celery<AMQPBroker>>, CeleryError> {
  celery::app!(
    broker = AMQPBroker { String::from( ampq ) },
    tasks = [ AMADEUS_INIT
            , cache::CONTEXT_CLEAR
            , cache::MODELS_REINIT
            //, bert::translation::EN2RU
            //, bert::translation::RU2EN
            , bert::chat::CHAT_GPT2
            , bert::qa::ASK
            , bert::neo::CHAT_NEO
            ],
    task_routes = [
      "*" => SALIERI_SERVICE,
    ],
  ).await
}

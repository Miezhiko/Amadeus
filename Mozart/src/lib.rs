#[macro_use] extern crate log;
#[macro_use] extern crate anyhow;
extern crate serde;

pub mod types;
pub mod cache;

#[cfg(not(target_os = "windows"))]
pub mod bert;

pub mod commands;
pub mod prelude;

use celery::{ Celery, self, prelude::* };

use std::sync::Arc;

pub const SALIERI_SERVICE: &str = "salieri";

// 5672 is default AMPQ port
pub const SALIERI_AMPQ: &str = "amqp://localhost:5672//";

#[celery::task]
pub async fn AMADEUS_INIT() -> TaskResult<()> {
  // init
  Ok(())
}

#[cfg(not(target_os = "windows"))]
pub async fn celery_init(ampq: &str) -> Result<Arc<Celery>, CeleryError> {
  celery::app!(
    broker = AMQPBroker { String::from( ampq ) },
    tasks = [ AMADEUS_INIT
            , cache::CONTEXT_CLEAR
            , cache::MODELS_REINIT
            , cache::REINIT_CACHE
            , cache::SET_CACHE
            , bert::chat::CHAT_GPT2
            , bert::qa::ASK
            , bert::neo::CHAT_NEO
            , bert::summarization::SUMMARIZE
            , bert::xlnet::XLNET
            , bert::code::CODEBERT
            ],
    task_routes = [
      "*" => SALIERI_SERVICE,
    ],
  ).await
}

#[cfg(target_os = "windows")]
pub async fn celery_init(ampq: &str) -> Result<Arc<Celery>, CeleryError> {
  celery::app!(
    broker = AMQPBroker { String::from( ampq ) },
    tasks = [ AMADEUS_INIT
            , cache::CONTEXT_CLEAR
            , cache::MODELS_REINIT
            , cache::REINIT_CACHE
            , cache::SET_CACHE
            ],
    task_routes = [
      "*" => SALIERI_SERVICE,
    ],
  ).await
}

#[macro_use] extern crate log;
extern crate serde;

use celery::{ Celery, self, prelude::* };

use std::sync::Arc;

pub const SALIERI: &str = "salieri";

// 5672 is default AMPQ port
pub const SALIERI_AMPQ: &str = "amqp://localhost:5672//";

#[celery::task]
pub async fn TASK(input_string: String) -> TaskResult<String> {
  let result = anyhow::Ok("success");
  match result {
    Ok(r) => {
      info!("receaved {input_string}");
      Ok(r.to_string())
    },
    Err(why) => {
      error!("something is going wrong {why}");
      Err( TaskError::ExpectedError( why.to_string() ) )
    }
  }
}

pub async fn celery_init(ampq: &str) -> Result<Arc<Celery<AMQPBroker>>, CeleryError> {
  celery::app!(
    broker = AMQPBroker { String::from( ampq ) },
    tasks = [ TASK ],
    task_routes = [
      "*" => SALIERI,
    ],
  ).await
}

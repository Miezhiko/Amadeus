use std::sync::Arc;
use tokio::{ runtime::Runtime, sync::Mutex};
use once_cell::sync::Lazy;
use celery::{ Celery, broker::AMQPBroker };

type SalieriBroker = Arc<Celery<AMQPBroker>>;

pub static SALIERI: Lazy<Mutex<Option<SalieriBroker>>> =
  Lazy::new(|| {
    Mutex::new(
      Runtime::new().unwrap().block_on(async {
        match mozart::celery_init(mozart::SALIERI_AMPQ).await {
          Ok(c) => Some(c),
          Err(why) => {
            error!("failed to connect to Salieri services: {why}");
            None
          }
        }
      })
    )
  });

pub async fn salieri_init() -> anyhow::Result<()> {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(mozart::TASK::new( "test".to_string() )).await?;
  }
  Ok(())
}

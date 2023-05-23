extern crate serde;

#[cfg(feature = "gpt4free")]
mod gpt4free;
#[cfg(feature = "gpt4free")]
mod opengpt;

#[cfg(feature = "kafka")]
mod kafka;

use env_logger::Env;

use mozart::{ celery_init, SALIERI_AMPQ, SALIERI_SERVICE };

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::Builder
            ::from_env(
              Env::default().default_filter_or("info")
            ).init();

  #[cfg(feature = "kafka")]
  kafka::run_with_workers(1);

  let salieri = celery_init(SALIERI_AMPQ).await?;
  salieri.display_pretty().await;
  salieri.consume_from(&[SALIERI_SERVICE]).await?;

  Ok(())
}

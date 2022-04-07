//#[macro_use] extern crate log;
extern crate serde;

use env_logger::Env;

use mozart::*;

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let my_app = celery_init(SALIERI_AMPQ).await?;
  my_app.display_pretty().await;
  my_app.consume_from(&[SALIERI_SERVICE]).await?;

  Ok(())
}

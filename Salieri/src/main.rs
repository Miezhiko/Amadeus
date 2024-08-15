use env_logger::Env;

use strauss::{ celery_init, SALIERI_AMPQ, SALIERI_SERVICE };

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::init_from_env(Env::default().default_filter_or("info"));

  let salieri = celery_init(SALIERI_AMPQ).await?;
  salieri.display_pretty().await;
  salieri.consume_from(&[SALIERI_SERVICE]).await?;

  Ok(())
}

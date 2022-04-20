extern crate serde;
use env_logger::Env;
use mozart::*;

#[cfg(not(target_os = "windows"))]
#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let salieri = celery_init(SALIERI_AMPQ).await?;
  salieri.display_pretty().await;

  salieri.consume_from(&[SALIERI_SERVICE]).await?;

  Ok(())
}

#[cfg(target_os = "windows")]
#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
  println!("Windows is not supported");
  Ok(())
}

extern crate serde;

mod gpt4free;
mod opengpt;

mod kafka;

use env_logger::Env;

#[tokio::main(worker_threads=8)]
async fn main() -> anyhow::Result<()> {
  env_logger::Builder
            ::from_env(
              Env::default().default_filter_or("info")
            ).init();

  kafka::run_with_workers(1);

  tokio::signal::ctrl_c().await?;

  Ok(())
}

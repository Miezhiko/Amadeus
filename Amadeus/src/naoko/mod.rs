mod options;
mod kafka;

use std::sync::Arc;

use serenity::prelude::*;

pub async fn naoko_init(ctx: &Arc<Context>) -> anyhow::Result<()> {
  // TODO: read once, save as once cell
  let iopts =
    options::get_ioptions()
            .map_err(|e| anyhow!("Failed to parse Dhall kafka config {e}"))?;

  kafka::run_with_workers(1, iopts, ctx);

  Ok(())
}

pub async fn naoko_request(text: &str) -> anyhow::Result<()> {
  // TODO: read once, save as once cell
  let iopts =
    options::get_ioptions()
            .map_err(|e| anyhow!("Failed to parse Dhall kafka config {e}"))?;

  kafka::request(iopts, args: &options::Args, text)

  Ok(())
}

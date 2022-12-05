use std::collections::BTreeMap;
use async_std::fs;

pub type Giveaway = BTreeMap<u64, f64>;

const GIVEAWAY_FILE_NAME_Y: &str  = "giveaway.yml";

pub async fn get_giveway() -> anyhow::Result<Giveaway> {
  let contents = fs::read_to_string(GIVEAWAY_FILE_NAME_Y).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn put_giveway(opts: &Giveaway) -> anyhow::Result<()> {
  let yml = serde_yaml::to_string(opts)?;
  fs::write(GIVEAWAY_FILE_NAME_Y, yml).await?;
  Ok(())
}

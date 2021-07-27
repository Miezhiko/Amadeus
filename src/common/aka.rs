use std::collections::HashMap;
use async_std::fs;

pub type Aka = HashMap<String, Option<String>>;

const AKA_FILE_NAME_Y: &str  = "aka.yml";

pub async fn get_aka() -> anyhow::Result<Aka> {
  let contents = fs::read_to_string(AKA_FILE_NAME_Y).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn put_aka(opts: &Aka) -> anyhow::Result<()> {
  let yml = serde_yaml::to_string(opts)?;
  fs::write(AKA_FILE_NAME_Y, yml).await?;
  Ok(())
}

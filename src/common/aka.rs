use std::collections::HashMap;
use async_std::fs;

pub type Aka = HashMap<String, Option<String>>;

const AKA_FILE_NAME: &str  = "aka.rs";

pub async fn get_aka() -> anyhow::Result<Aka> {
  let contents = fs::read_to_string(AKA_FILE_NAME).await?;
  let rdn = rudano::from_str(&contents)?;
  Ok(rdn)
}

pub async fn put_aka(opts: &Aka) -> anyhow::Result<()> {
  let rdn = rudano::to_string_pretty(opts)?;
  fs::write(AKA_FILE_NAME, rdn).await?;
  Ok(())
}

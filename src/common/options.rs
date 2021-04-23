use crate::types::options::{ IOptions, ROptions };

use async_std::fs;

const DHALL_FILE_NAME: &str = "conf.dhall";
const RUDANO_FILE_NAME: &str  = "conf.rs";

pub fn get_ioptions() -> Result<IOptions, serde_dhall::Error> {
  serde_dhall::from_file(DHALL_FILE_NAME).parse()
}

pub async fn get_roptions() -> anyhow::Result<ROptions> {
  let contents = fs::read_to_string(RUDANO_FILE_NAME).await?;
  let rdn = rudano::from_str(&contents)?;
  Ok(rdn)
}

pub async fn put_roptions(opts: &ROptions) -> anyhow::Result<()> {
  let rdn = rudano::to_string_pretty(opts)?;
  fs::write(RUDANO_FILE_NAME, rdn).await?;
  Ok(())
}

use crate::types::options::{ IOptions, ROptions };

use async_std::fs;

const DHALL_FILE_NAME: &str = "conf.dhall";
const YAML_FILE_NAME: &str  = "conf.yml";

//TODO: problem on serde dhall
#[allow(clippy::result_large_err)]
pub fn get_ioptions() -> Result<IOptions, serde_dhall::Error> {
  serde_dhall::from_file(DHALL_FILE_NAME).parse()
}

pub async fn get_roptions() -> anyhow::Result<ROptions> {
  let contents = fs::read_to_string(YAML_FILE_NAME).await?;
  let yml = serde_yaml::from_str(&contents)?;
  Ok(yml)
}

pub async fn put_roptions(opts: &ROptions) -> anyhow::Result<()> {
  let yml = serde_yaml::to_string(opts)?;
  fs::write(YAML_FILE_NAME, yml).await?;

  Ok(())
}

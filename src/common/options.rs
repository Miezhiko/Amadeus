use crate::common::types::{ IOptions, ROptions };

use async_std::{ fs, io::Error };

const DHALL_FILE_NAME: &'static str = "conf.dhall";
const JSON_FILE_NAME: &'static str = "conf.json";

pub fn get_ioptions() -> Result<IOptions, serde_dhall::Error> {
  serde_dhall::from_file(DHALL_FILE_NAME).parse()
}

pub async fn get_roptions() -> Result<ROptions, Error> {
  let contents = fs::read_to_string(JSON_FILE_NAME).await?;
  let j = serde_json::from_str(contents.as_str())?;
  Ok(j)
}

pub async fn put_roptions(opts : &ROptions) -> Result<(), Error> {
  let j = serde_json::to_string_pretty(opts)?;
  fs::write(JSON_FILE_NAME, j).await?;
  Ok(())
}

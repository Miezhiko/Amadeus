const DHALL_FILE_NAME: &str = "kafka.dhall";

pub struct Args {
  pub command: Option<String>,
  pub channel: u64,
  pub user: u64,
  pub msg: u64
}

#[derive(Clone, Deserialize, Debug)]
pub struct IOptions {
  pub kafka_address: String,
  pub kafka_group: String,
  pub kafka_sink: String,
  pub kafka_target: String
}

#[allow(clippy::result_large_err)]
pub fn get_ioptions() -> Result<IOptions, serde_dhall::Error> {
  serde_dhall::from_file(DHALL_FILE_NAME).parse()
}

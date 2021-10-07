use crate::types::common::IServer;

#[derive(Clone, Deserialize, Debug)]
pub struct IOptions {
  pub discord: String,
  pub app_id: u64,
  pub guild: u64,
  pub amadeus_guild: u64,
  pub servers: Vec<IServer>,
  pub twitch_client_id: String,
  pub twitch_client_secret: String,
  pub tenor_key: String,
  pub flo_secret: String,
  pub lazy_static_models: bool
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ROptions {
  pub rejoin: bool,
  pub last_guild: u64,
  pub last_channel: u64,
  pub last_stream: String,
  pub twitch: String
}

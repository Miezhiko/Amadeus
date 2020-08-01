#[derive(Debug, Clone, Deserialize)]
pub struct IOptions {
  pub discord: String,
  pub guild: u64,
  pub twitch_oauth: String,
  pub twitch_client_id: String,
  pub twitch_client_secret: String,
  pub tenor_key: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROptions {
  pub rejoin: bool,
  pub last_guild: u64,
  pub last_channel: u64,
  pub last_stream: String
}

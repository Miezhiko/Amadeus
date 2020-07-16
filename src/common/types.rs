#[derive(Debug, Clone, Deserialize)]
pub struct IOptions {
  pub discord: String,
  pub guild: u64,
  pub twitch_oauth: String,
  pub twitch_client_id: String,
  pub twitch_client_secret: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ROptions {
  pub rejoin: bool,
  pub last_guild: u64,
  pub last_channel: u64,
  pub last_stream: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct Streams {
  pub ggru: Option<String>,
  pub twitch: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Player {
  pub battletag: String,
  pub discord: u64,
  pub streams: Option<Streams>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reaction {
  pub id: u64,
  pub name: String
}

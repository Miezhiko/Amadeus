pub const BLACKLIST_JSON: &str = "blacklist.json";

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Blacklisted {
  pub battletag: String,
  pub reason: String
}

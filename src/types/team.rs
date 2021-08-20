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
pub struct DiscordServer {
  pub uid: u64,
  pub players: Vec<Player>
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordPlayer {
  pub player: Player,
  pub discords: Vec<u64>
}

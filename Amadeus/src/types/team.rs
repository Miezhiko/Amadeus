use std::collections::HashMap;

use serenity::model::id::ChannelId;

#[derive(Debug, Clone, Deserialize)]
pub struct Streams {
  pub ggru: Option<String>,
  pub twitch: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Player {
  pub battletag: String,
  pub discord: u64,
  pub streams: Option<Streams>,
  pub other_acc: Vec<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordServer {
  pub uid: u64,
  pub games: Option<u64>,
  pub games2: Option<u64>,
  pub games4: Option<u64>,
  pub streams: Option<u64>,
  pub events: Option<u64>,
  pub log: Option<u64>,
  pub players: Vec<Player>
}

#[derive(Debug, Clone)]
pub struct DiscordFields {
  pub games: Option<u64>,
  pub games2: Option<u64>,
  pub games4: Option<u64>,
  pub streams: Option<u64>,
  pub events: Option<ChannelId>,
  pub log: Option<ChannelId>,
}

pub type Discords = HashMap<u64, DiscordFields>;

#[derive(Debug, Clone)]
pub struct DiscordPlayer {
  pub player: Player,
  pub discords: Vec<u64>
}

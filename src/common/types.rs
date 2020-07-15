#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct Streams {
  pub ggru: Option<&'static str>,
  pub twitch: Option<&'static str>
}

#[derive(Debug, Clone)]
pub struct Player {
  pub battletag: &'static str,
  pub discord: u64,
  pub streams: Option<Streams>
}

pub const fn player(battletag: &'static str, discord: u64) -> Player {
  Player {
    battletag: battletag,
    discord: discord,
    streams: None
  }
}

pub const fn streamer(battletag: &'static str, discord: u64, streams: Option<Streams>) -> Player {
  Player {
    battletag: battletag,
    discord: discord,
    streams: streams
  }
}

pub const fn streams(ggru: &'static str, twitch: &'static str) -> Option<Streams> {
  Some(Streams {
    ggru: Some(ggru),
    twitch: Some(twitch)
  })
}

pub const fn twitch(twitch: &'static str) -> Option<Streams> {
  Some(Streams {
    ggru: None,
    twitch: Some(twitch)
  })
}

#[allow(dead_code)]
pub const fn ggru(ggru: &'static str) -> Option<Streams> {
  Some(Streams {
    ggru: Some(ggru),
    twitch: None
  })
}

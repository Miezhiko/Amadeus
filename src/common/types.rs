#[derive(Debug, Clone)]
pub struct AOptions {
  pub rejoin: bool,
  pub discord: String,
  pub last_guild: String,
  pub last_channel: String,
  pub last_stream: String,
  pub twitch_oauth: String,
  pub twitch_client_id: String,
  pub twitch_client_secret: String
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

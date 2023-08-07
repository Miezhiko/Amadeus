use crate::types::team::DiscordPlayer;

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum GameMode {
  Solo,
  Team2,
  Team4
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Winner {
  pub player: (String, u64),
  pub won: bool,
  pub mmr: u32,
  pub race: u32
}

#[allow(non_snake_case)]
pub struct FinishedGame {
  pub desc: Vec<String>,
  pub passed_time: u32,
  pub link: String,
  pub winners: Vec<Winner>,
  pub additional_fields: Option<(String, String, String, String)>,
  pub hero_png: Option<String>
}

#[allow(non_snake_case)]
pub struct Bet {
  pub guild: u64,
  pub member: u64,
  pub points: u64,
  pub positive: bool,
  pub registered: bool
}

#[allow(non_snake_case)]
pub struct TrackingGame {
  pub tracking_msg_id: Vec<(u64, u64)>,
  pub passed_time: u32,
  pub still_live: bool,
  pub players: Vec<DiscordPlayer>,
  pub bets: Vec<Bet>,
  pub fails: u32,
  pub mode: GameMode,
  pub flo_tv: Option<String>
}

#[allow(non_snake_case)]
pub struct StartingGame<'a> {
  pub key: String,
  pub description: Vec<String>,
  pub players: Vec<&'a DiscordPlayer>,
  pub host: String,
  pub mode: GameMode
}

#[derive(Default)]
#[allow(non_snake_case)]
pub struct W3CStats {
  pub games_solo: usize,
  pub games_2x2: usize,
  pub games_4x4: usize
}

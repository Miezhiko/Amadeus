use crate::types::team::Player;

#[allow(non_snake_case)]
pub struct FinishedGame {
  pub desc: Vec<String>,
  pub passed_time: u32,
  pub link: String,
  pub winners: Vec<(u64, bool)>,
  pub additional_fields: Option<(String, String, String, String)>,
  pub hero_png: Option<String>
}

#[allow(non_snake_case)]
pub struct Bet {
  pub guild: u64,
  pub member: u64,
  pub points: u64,
  pub positive: bool
}

#[allow(non_snake_case)]
pub struct TrackingGame {
  pub tracking_msg_id: u64,
  pub passed_time: u32,
  pub still_live: bool,
  pub players: Vec<Player>,
  pub bets: Vec<Bet>,
  pub fails: u32
}

#[allow(non_snake_case)]
pub struct StartingGame {
  pub key: String,
  pub description: Vec<String>,
  pub players: Vec<Player>
}

use crate::types::team::Player;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct FinishedGame {
  pub desc: Vec<String>,
  pub passed_time: u32,
  pub link: String,
  pub winners: Vec<(u64, bool)>,
  pub additional_fields: Option<(String, String, String, String)>,
  pub hero_png: Option<String>
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct TrackingGame {
  pub tracking_msg_id: u64,
  pub passed_time: u32,
  pub still_live: bool,
  pub players: Vec<Player>
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct StartingGame {
  pub key: String,
  pub description: Vec<String>,
  pub players: Vec<Player>
}

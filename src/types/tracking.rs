use crate::types::team::Player;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct FinishedGame {
  pub desc: String,
  pub passed_time: u32,
  pub win: bool,
  pub additional_fields: Option<(String, String, String, String)>,
  pub hero_png: Option<String>
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct TrackingGame {
  pub tracking_msg_id: u64,
  pub passed_time: u32,
  pub still_live: bool,
  pub player: Player
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct StartingGame {
  pub key: String,
  pub description: String,
  pub player: Player
}

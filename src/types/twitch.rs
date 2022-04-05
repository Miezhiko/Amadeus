pub static TWITCH_WC3: &str = "12924";

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TwitchData {
  pub id: String,
  pub user_id: String,
  pub user_login: Option<String>,
  pub user_name: String,
  pub game_id: String, // 12924 = wc3
  pub game_name: Option<String>, // "Warcraft III"
  #[serde(rename = "type")]
  pub type_string: String,
  pub title: String,
  pub viewer_count: u32,
  pub started_at: String,
  pub language: String,
  pub thumbnail_url: String,
  pub tag_ids: Option<Vec<String>>,
  pub is_mature: bool // Twitch allows 18+ content?
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Pagination {}

pub_struct!(Twitch {
  data: Vec<TwitchData>,
  pagination: Pagination,
});

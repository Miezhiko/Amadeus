#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TwitchData {
  pub id: String,
  pub user_id: String,
  pub user_name: String,
  pub game_id: String, // 12924 = wc3?
  #[serde(rename = "type")]
  pub type_string: String,
  pub title: String,
  pub viewer_count: u32,
  pub started_at: String,
  pub language: String,
  pub thumbnail_url: String,
  pub tag_ids: Vec<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Pagination {
}

pub_struct!(Twitch {
  data: Vec<TwitchData>,
  pagination: Pagination,
});

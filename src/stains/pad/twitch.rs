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

/*
{"data":
[{"id":"1841817985","user_id":"44219100",
"user_name":"SQKTGW",
"game_id":"12924","type":"live",
"title":"вечное сияние чистой скиллухи",
"viewer_count":10,"started_at":"2020-06-04T13:10:36Z",
"language":"ru",
"thumbnail_url":"https://static-cdn.jtvnw.net/previews-ttv/live_user_sqktgw-{width}x{height}.jpg",
"tag_ids":["0569b171-2a2b-476e-a596-5bdfb45a1327"]}],
"pagination":{}}

{"data":[],"pagination":{}}
*/

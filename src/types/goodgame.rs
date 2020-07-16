#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GGGame {
  pub title: String,
  pub url: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GGChannel {
  pub id: u32,
  pub key: String,
  pub premium: String,
  pub title: String,
  pub max_viewers: u32,
  pub player_type: String,
  pub gg_player_src: String,
  pub embed: String,
  pub img: String,
  pub thumb: String,
  pub description: String,
  pub adult: bool,
  pub hidden: bool,
  pub games: Vec<GGGame>,
  pub url: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GGSelf {
  pub href: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GGLinks {
  #[serde(rename = "self")]
  pub self_link: GGSelf
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct GoodGameData {
  pub request_key: String,
  pub id: u32,
  pub key: String,
  pub is_broadcast: bool,
  pub broadcast_started: u32,
  pub broadcast_end: u32,
  pub url: String,
  pub status: String,
  pub viewers: String,
  pub player_viewers: String,
  pub users_in_chat: String,
  pub channel: GGChannel,
  pub _links: GGLinks
}

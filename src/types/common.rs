use serenity::prelude::*;

use std::{ collections::HashMap, sync::Arc };
use reqwest::Client as Reqwest;

#[derive(Debug, Clone, Deserialize)]
pub struct Reaction {
  pub id: u64,
  pub name: String
}

pub struct PubCreds;

impl TypeMapKey for PubCreds {
  type Value = Arc<HashMap<String, String>>;
}

pub struct CoreGuilds;
pub struct AllGuilds;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum CoreGuild {
  UserId,
  Amadeus,
  HEmo
}

impl TypeMapKey for CoreGuilds {
  type Value = Arc<HashMap<CoreGuild, u64>>;
}

impl TypeMapKey for AllGuilds {
  type Value = Arc<Vec<u64>>;
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
  type Value = Arc<Reqwest>;
}

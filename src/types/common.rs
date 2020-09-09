use serenity::prelude::*;

use std::{ collections::HashMap, sync::Arc };
use reqwest::Client as Reqwest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum CoreGuild {
  HEmo,
  Storage,
  Safe,
  Unsafe
}

#[derive(Debug, Clone, Deserialize)]
pub struct IServer {
  pub id: u64,
  pub name: String,
  pub kind: CoreGuild
}

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

impl TypeMapKey for CoreGuilds {
  type Value = Arc<HashMap<CoreGuild, u64>>;
}

impl TypeMapKey for AllGuilds {
  type Value = Arc<Vec<IServer>>;
}

pub struct ReqwestClient;

impl TypeMapKey for ReqwestClient {
  type Value = Arc<Reqwest>;
}

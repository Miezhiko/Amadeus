use serenity::prelude::*;

use std::{ collections::HashMap, sync::Arc };
use reqwest::Client as Reqwest;

#[derive(Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum ChannelLanguage {
  English,
  Russian,
  Bilingual
}

#[derive(Clone, Deserialize)]
pub struct LChannel {
  pub id: u64,
  pub lang: ChannelLanguage,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Debug)]
pub enum CoreGuild {
  Safe,
  Unsafe,
  HEmo,
  Storage,
  Amadeus
}

#[derive(Clone, Copy, Deserialize, Debug)]
pub struct IServer {
  pub id: u64,
  pub kind: CoreGuild
}

#[derive(Clone, Deserialize)]
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

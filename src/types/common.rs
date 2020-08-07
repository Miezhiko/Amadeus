use serenity::prelude::*;

use std::{ collections::HashMap, sync::Arc };

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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum CoreGuild {
  Amadeus,
  HEmo,
}

impl TypeMapKey for CoreGuilds {
  type Value = Arc<HashMap<CoreGuild, u64>>;
}

use crate::{
  common::{
    msg::{ channel_message }
  }
};

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  },
};

use wikibase::SearchResults;
use wikibase::query::SearchQuery;

#[command]
pub fn wiki(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
  let input = args.message();

  let mut configuration = wikibase::Configuration::new("Amadeus/1.0").unwrap();
  configuration.set_api_url("https://liquipedia.net/warcraft/api.php");

  let entity_type = wikibase::EntityType::Item;
  let search_query = SearchQuery::new(input, "en", "en", 20, entity_type);

  let mut out : Vec<String> = Vec::new();

  match SearchResults::new_from_query(&search_query, &configuration) {
    Ok(value) => {
      for result in value.results() {
        out.push( result.label().value().to_string() );
      }
    },
    Err(error) => error!("{:?}", error)
  };

  if out.len() > 0 {
    let out_str = out.join("\n");
    channel_message(&ctx, &msg, out_str.as_str());
  }
  Ok(())
}

use crate::{ collections::{ base::GREETINGS
                          , channels::IGNORED }
           , common::constants::PREFIX
           , common::i18n::{ help_i18n, US_ENG }
           , steins::ai::response
           };

use serenity::{
  prelude::*,
  framework::standard::{ Args
                       , CommandResult
                       , macros::{ hook, help }
                       , HelpOptions, CommandGroup, help_commands },
  model::{ channel::Message, id::UserId }
};

use std::collections::HashSet;

use regex::Regex;
use once_cell::sync::Lazy;

use rand::{ rngs::StdRng
          , seq::SliceRandom
          , SeedableRng };

#[hook]
pub async fn before(_ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
  if IGNORED.contains(&msg.channel_id.0) {
    return false;
  }
  debug!("Running command: {}, Message: {}", &cmd_name, &msg.content);
  true
}

#[hook]
pub async fn after( ctx: &Context
                  , msg: &Message
                  , cmd_name: &str
                  , error: CommandResult ) {
  if let Err(why) = &error {
    error!("Error while running command {}", &cmd_name);
    error!("{:?}", &error);
    if let Err(why) = msg.channel_id.say(ctx, &why).await {
      error!("Unable to send messages on channel {} {why}", &msg.channel_id.0);
    }
  }
}

fn greeting_regex_from_str(c: &str) -> Option<Regex> {
  let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
  Regex::new(&regex).ok()
}

#[hook]
pub async fn unrecognised_command( ctx: &Context
                                 , msg: &Message
                                 , _command_name: &str ) {
  if msg.content.starts_with(PREFIX) {
    // Don't chat with prefix like ~hell how are you
    // Only reply if was mentioned
    return;
  }
  static GREETINGS_CHECKS: Lazy<Vec<Regex>> =
    Lazy::new(||
      GREETINGS.iter().filter_map(|c|
        greeting_regex_from_str(c)
      ).collect()
    );
  if GREETINGS_CHECKS.iter().any(|r| r.is_match(&msg.content)) {
    let mut rng = StdRng::from_entropy();
    if let Some(hi_reply) = GREETINGS.choose(&mut rng) {
      if let Err(why) = msg.reply(&ctx, hi_reply).await {
        error!("Error sending greeting reply: {why}");
      }
    }
  } else {
    response::response(ctx, msg).await;
  }
}

#[help]
#[individual_command_tip = "Amadeus"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(9)]
#[wrong_channel = "Nothing"]
#[group_prefix = "Prefix commands"]
pub async fn help_command( ctx: &Context
                         , msg: &Message
                         , args: Args
                         , help_options: &'static HelpOptions
                         , groups: &[&'static CommandGroup]
                         , owners: HashSet<UserId> ) -> CommandResult {
  if args.is_empty() {
    help_i18n(ctx, msg, &US_ENG).await;
  } else  { help_commands::with_embeds( ctx, msg, args
                                      , help_options, groups, owners
                                      ).await?; }
  Ok(())
}

use crate::{ collections::{ base::GREETINGS
                          , channels::IGNORED }
           , common::i18n::{ help_i18n, US_ENG }
           , steins::ai::chain
           , types::serenity::IContext
           };

use serenity::{
  prelude::*,
  framework::standard::{ DispatchError, Args
                       , Reason, CommandResult
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
pub async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
  match error {
    // Notify the user if the reason of the command failing to execute was because of
    // inssufficient arguments.
    DispatchError::NotEnoughArguments { min, given } => {
      let s = {
        if given == 0  && min == 1{
          "I need an argument to run this command".to_string()
        } else if given == 0 {
          format!("I need atleast {} arguments to run this command", min)
        } else {
          format!("I need {} arguments to run this command, but i was only given {}.", min, given)
        }
      };
      // Send the message, but supress any errors that may occur.
      let _ = msg.channel_id.say(ctx, s).await;
    },
    DispatchError::CheckFailed(_, reason) => {
      if let Reason::User(r) = reason {
        let _ = msg.channel_id.say(ctx, r).await;
      }
    },
    DispatchError::Ratelimited(x) => {
      let _ = msg.reply(ctx, format!("You can't run this command for {:#?} more.", x)).await;
    }
    // eprint prints to stderr rather than stdout.
    _ => {
      error!("Unhandled dispatch error: {:?}", error);
      eprintln!("An unhandled dispatch error has occurred:");
      eprintln!("{:?}", error);
    }
  }
}

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
      error!("Unable to send messages on channel {} {:?}", &msg.channel_id.0, why);
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
        error!("Error sending greeting reply: {:?}", why);
      }
    }
  } else {
    let lsm = {
      let data = ctx.data.read().await;
      if let Some(icontext) = data.get::<IContext>() {
        *icontext
      } else { false }
    };
    chain::response(&ctx, &msg, lsm).await;
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
  } else if help_commands::with_embeds( ctx, msg, args
                                      , help_options, groups, owners
                                      ).await.is_none() {
    warn!("empty help answer");
  }
  Ok(())
}

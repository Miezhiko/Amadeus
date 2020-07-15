use crate::{
  stains::ai::chain,
  common::options, common::types::IOptions,
  handler::Handler,
  commands::{
    meta::*,
    chat::*,
    voice::*,
    warcraft::*,
    pad::*,
    owner::*,
    admin::*
  },
  collections::base::{ GREETINGS }
};

use serenity::{
  prelude::*,
  framework::StandardFramework,
  framework::standard::{
    DispatchError, Args, CommandOptions, CheckResult,
    Reason, CommandResult,
    macros::{group, check, hook}
  },
  model::{channel::{Message}}
};

use argparse::{
  ArgumentParser,
  action::{IFlagAction, ParseResult}
};

use env_logger::Env;

use std::collections::HashSet;
use std::sync::Arc;

use regex::Regex;

use rand::{
  rngs::StdRng,
  seq::SliceRandom,
  SeedableRng
};

pub struct Version();

impl IFlagAction for Version {
  fn parse_flag(&self) -> ParseResult {
    set!( version = env!("CARGO_PKG_VERSION").to_string()
        , pname = "Amadeus"
        , version_string = format!("{} {}", pname, version) );
    println!("{}", version_string);
    return ParseResult::Exit;
  }
}

#[group]
#[commands(ping, help, embed, qrcode, urban)]
struct Meta;

#[group]
#[commands(quote, score, give)]
struct Chat;

#[group]
#[commands(join, leave, play, repeat)]
struct Voice;

#[group]
#[commands(lineup, yesterday, today, tomorrow, weekends)]
struct Warcraft;

#[group]
#[commands(stats, ongoing)]
struct Pad;

#[group]
#[owners_only]
#[checks(Admin)]
#[commands(say, set, clear)]
struct Owner;

#[group]
#[checks(Admin)]
#[commands(idle, stream)]
struct Admin;

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
async fn admin_check(ctx: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
  if let Some(member) = msg.member(&ctx.cache).await {
    if let Ok(permissions) = member.permissions(&ctx.cache).await {
      return permissions.administrator().into();
    }
  }
  false.into()
}

#[hook]
async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
  match error {
    // Notify the user if the reason of the command failing to execute was because of
    // inssufficient arguments.
    DispatchError::NotEnoughArguments { min, given } => {
      let s = {
        if given == 0  && min == 1{
          format!("I need an argument to run this command")
        } else if given == 0 {
          format!("I need atleast {} arguments to run this command", min)
        } else {
          format!("I need {} arguments to run this command, but i was only given {}.", min, given)
        }
      };
      // Send the message, but supress any errors that may occur.
      let _ = msg.channel_id.say(ctx, s).await;
    },
    DispatchError::IgnoredBot {} => {
        return;
    },
    DispatchError::CheckFailed(_, reason) => {
      if let Reason::User(r) = reason {
        let _ = msg.channel_id.say(ctx, r).await;
      }
    },
    DispatchError::Ratelimited(x) => {
      let _ = msg.reply(ctx, format!("You can't run this command for {} more seconds.", x)).await;
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
async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: CommandResult) {
  // error is the command result.
  // inform the user about an error when it happens.
  if let Err(why) = &error {
    error!("Error while running command {}", &cmd_name);
    error!("{:?}", &error);
    if let Err(_) = msg.channel_id.say(ctx, &why).await {
      error!("Unable to send messages on channel id {}", &msg.channel_id.0);
    };
  }
}

#[hook]
async fn unrecognised_command(ctx: &Context, msg: &Message, _command_name: &str) {
  if let Some(_) = GREETINGS.iter().find(|c| {
    let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
    let is_greeting = Regex::new(regex.as_str()).unwrap();
    is_greeting.is_match(msg.content.as_str()) }) 
  {
    let mut rng = StdRng::from_entropy();
    set! { hi_reply = GREETINGS.choose(&mut rng).unwrap()
        , reply = format!("{}", hi_reply) };
    if let Err(why) = msg.reply(&ctx, reply).await {
      error!("Error sending greeting reply: {:?}", why);
    }
  } else {
    chain::response(&ctx, &msg).await;
  }
}

pub async fn run(opts : &IOptions) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  { // this block limits scope of borrows by ap.refer() method
    let mut ap = ArgumentParser::new();
    let pname = "Amadeus";
    ap.set_description(pname);
    ap.add_option(&["--version"], Version(), "Show version");
    ap.parse_args_or_exit();
  }

  let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "info")
    .write_style_or("MY_LOG_STYLE", "always");

  env_logger::init_from_env(env);

  info!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());

  let http = serenity::http::Http::new_with_token(&opts.discord);

  info!("http context created");

  // Obtains and defines the owner/owners of the Bot Application
  // and the bot id. 
  let (owners, bot_id) = match http.get_current_application_info().await {
    Ok(info) => {
      let mut owners = HashSet::new();
      owners.insert(info.owner.id);
      (owners, info.id)
    },
    Err(why) => panic!("Could not access application info: {:?}", why),
  };

  info!("application info loaded");

  let runtime_options = options::get_roptions().await?;

  info!("all the options loaded");

  let std_framework =
    StandardFramework::new()
     .configure(|c| c
      .owners(owners)
      .on_mention(Some(bot_id))
      .prefix("~")
      .delimiters(vec![" "])
      .case_insensitivity(true))
      .on_dispatch_error(on_dispatch_error)
      .after(after)
      .unrecognised_command(unrecognised_command)
      .group(&META_GROUP)
      .group(&CHAT_GROUP)
      .group(&VOICE_GROUP)
      .group(&WARCRAFT_GROUP)
      .group(&PAD_GROUP)
      .group(&OWNER_GROUP)
      .group(&ADMIN_GROUP);

  let mut client = serenity::Client::new(&opts.discord)
                    .event_handler(Handler::new(opts, runtime_options))
                    .framework(std_framework).await?;
  {
    let mut data = client.data.write().await;
    data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
  }

  // start listening for events by starting a single shard
  if let Err(why) = client.start_autosharded().await {
    eprintln!("An error occurred while running the client: {:?}", why);
  }

  Ok(())
}

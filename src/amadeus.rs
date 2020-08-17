use crate::{
  types::{
    common::{
      PubCreds, CoreGuild, CoreGuilds, AllGuilds
    },
    options::IOptions
  },
  stains::ai::chain,
  common::options,
  handler::Handler,
  commands::{
    meta::*,
    chat::*,
    voice::*,
    warcraft::*,
    pad::*,
    owner::*,
    admin::*,
    tictactoe::*,
    images::*,
    tranlation::*
  },
  collections::base::GREETINGS
};

use serenity::{
  prelude::*,
  framework::StandardFramework,
  framework::standard::{
    DispatchError, Args, CommandOptions, CheckResult,
    Reason, CommandResult,
    macros::{ group, check, hook }
  },
  model::channel::Message
};

use argparse::{
  ArgumentParser,
  action::{IFlagAction, ParseResult}
};

use env_logger::Env;

use std::collections::{ HashSet, HashMap };
use std::sync::Arc;

use regex::Regex;
use reqwest::Client as Reqwest;

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
    ParseResult::Exit
  }
}

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

#[group]
#[commands(info, help, help_ru, version, embed, qrcode, urban, uptime, tic_tac_toe, changelog)]
struct Meta;

#[group]
#[commands(quote, boris, owo, score, top, give)]
struct Chat;

#[group]
#[commands(en2ru, ru2en, en2de, de2en, en2fr, fr2en)]
struct Translate;

#[group]
#[commands(cry, hug, pat, slap, cringe, wave, sex, ahegao, clap, shrug, gifsearch
  , lol, angry, dance, confused, shock, nervous, sad, happy, annoyed, omg, smile
  , ew, awkward, oops, lazy, hungry, srtessed, scared, bored, yes, no, bye, sorry
  , sleepy, wink, facepalm, whatever)]
struct Images;

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
#[commands(say, set, clear, upgrade)]
struct Owner;

#[group]
#[checks(Admin)]
#[commands(idle, stream, give_win, register_lose, mute, unmute, blacklist)]
struct Admin;

#[hook]
async fn on_dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
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
    if let Err(why) = msg.channel_id.say(ctx, &why).await {
      error!("Unable to send messages on channel {} {:?}", &msg.channel_id.0, why);
    }
  }
}

#[hook]
async fn unrecognised_command(ctx: &Context, msg: &Message, _command_name: &str) {
  let is_valid_greeting = |c| {
    let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
    let is_greeting = Regex::new(&regex).unwrap();
    is_greeting.is_match(&msg.content) };
  if GREETINGS.iter().any(is_valid_greeting) {
    let mut rng = StdRng::from_entropy();
    if let Some(hi_reply) = GREETINGS.choose(&mut rng) {
      if let Err(why) = msg.reply(&ctx, hi_reply).await {
        error!("Error sending greeting reply: {:?}", why);
      }
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

  let (owners, amadeus_id) = match http.get_current_application_info().await {
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

  // TODO: maybe use it instead of passing options for twitch and things (things)
  let mut creds = HashMap::new();
  creds.insert("tenor".to_string(), opts.tenor_key.clone());

  let mut core_guilds = HashMap::new();
  core_guilds.insert(CoreGuild::UserId, *amadeus_id.as_u64());
  core_guilds.insert(CoreGuild::Amadeus, opts.amadeus_guild);
  core_guilds.insert(CoreGuild::HEmo, opts.guild);

  let std_framework =
    StandardFramework::new()
     .configure(|c| c
      .owners(owners)
      .on_mention(Some(amadeus_id))
      .prefix("~")
      .delimiters(vec![" "])
      .case_insensitivity(true))
      .on_dispatch_error(on_dispatch_error)
      .after(after)
      .unrecognised_command(unrecognised_command)
      .group(&META_GROUP)
      .group(&CHAT_GROUP)
      .group(&TRANSLATE_GROUP)
      .group(&IMAGES_GROUP)
      .group(&VOICE_GROUP)
      .group(&WARCRAFT_GROUP)
      .group(&PAD_GROUP)
      .group(&OWNER_GROUP)
      .group(&ADMIN_GROUP);

  let mut client = serenity::Client::new(&opts.discord)
                    .event_handler(Handler::new( opts
                                               , runtime_options
                                               , amadeus_id
                                               )
                                  )
                    .framework(std_framework).await?;
  {
    let mut data = client.data.write().await;
    data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    data.insert::<ReqwestClient>(Arc::new(Reqwest::new()));
    data.insert::<PubCreds>(Arc::new(creds));
    data.insert::<CoreGuilds>(Arc::new(core_guilds));
    data.insert::<AllGuilds>(Arc::new(opts.servers.clone()));
  }

  // start listening for events by starting a single shard
  if let Err(why) = client.start_autosharded().await {
    eprintln!("An error occurred while running the client: {:?}", why);
  }

  Ok(())
}

use crate::{
  types::{
    common::{ PubCreds
            , ReqwestClient
            , CoreGuild
            , CoreGuilds
            , IServer
            , AllGuilds },
    options::IOptions
  },
  steins::ai::chain,
  common::{ options,
    i18n::{ help_i18n, US_ENG }
  },
  handler::Handler,
  commands::{ meta::*
            , chat::*
            , warcraft::*
            , pad::*
            , owner::*
            , admin::*
            , tictactoe::*
            , images::*
            , tranlation::*
            , lilyal::*
            , bets::* },
  collections::{ base::GREETINGS
               , channels::IGNORED }
};

use serenity::{
  prelude::*,
  framework::StandardFramework,
  framework::standard::{
    DispatchError, Args, CommandOptions, CheckResult,
    Reason, CommandResult,
    macros::{ group, check, hook, help },
    HelpOptions, CommandGroup, help_commands
  },
  model::{
    channel::Message,
    id::UserId
  }
};

use argparse::{
  ArgumentParser,
  action::{IFlagAction, ParseResult}
};

use tracing::{ Level, instrument };
use tracing_subscriber::FmtSubscriber;
use tracing_log::LogTracer;

use std::collections::{ HashSet, HashMap };
use std::sync::Arc;

use regex::Regex;
use reqwest::Client as Reqwest;

use rand::{ rngs::StdRng
          , seq::SliceRandom
          , SeedableRng };

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
async fn admin_check( ctx: &Context
                    , msg: &Message
                    , _: &mut Args
                    , _: &CommandOptions ) -> CheckResult {
  if let Ok(member) = msg.member(ctx).await {
    if let Ok(permissions) = member.permissions(&ctx.cache).await {
      return permissions.administrator().into();
    }
  }
  false.into()
}

#[group("Meta")]
#[description = "Basic commands"]
#[commands( info, version, embed, qrcode, urban, uptime, tic_tac_toe, changelog
          , join, leave, play, repeat , help_ru, time )]
struct Meta;

#[group("Chat")]
#[description = "Chat commands"]
#[commands(quote, boris, owo, score, top, give, feminize, extreme_feminize)]
struct Chat;

#[group("Translation")]
#[description = "Translation commands"]
#[commands(en2ru, ru2en, en2de, de2en, en2fr, fr2en)]
struct Translate;

#[group("Images")]
#[description = "Gifs posting"]
#[commands(cry, hug, pat, slap, cringe, wave, sex, ahegao, clap, shrug, gifsearch
  , lol, angry, dance, confused, shock, nervous, sad, happy, annoyed, omg, smile
  , ew, awkward, oops, lazy, hungry, stressed, scared, bored, yes, no, bye, sorry
  , sleepy, wink, facepalm, whatever, pout, smug, smirk)]
struct Images;

#[group("Warcraft")]
#[description = "Warcraft events"]
#[commands(lineup, yesterday, today, tomorrow, weekends)]
struct Warcraft;

#[group("W3C")]
#[description = "w3champions commands"]
#[commands(stats, ongoing, veto, bet)]
struct Pad;

#[group("Database")]
#[description = "Information storage commands"]
#[commands(register, show, delete)]
struct Lilyal;

#[group("Owner")]
#[help_available(false)]
#[owners_only]
#[checks(Admin)]
#[commands(say, set, clear, upgrade
  , update_cache, clear_chain_cache
  , twitch_token_update)]
struct Owner;

#[group("Admin")]
#[checks(Admin)]
#[help_available(false)]
#[commands(idle, stream, give_win, register_lose, mute, unmute)]
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
async fn before(_ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
  if IGNORED.contains(&msg.channel_id.0) {
    return false;
  }

  debug!("Running command: {}, Message: {}", &cmd_name, &msg.content);

  true
}

#[hook]
async fn after( ctx: &Context
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

#[hook]
async fn unrecognised_command( ctx: &Context
                             , msg: &Message
                             , _command_name: &str ) {
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

#[help]
#[individual_command_tip = "Amadeus"]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(9)]
#[wrong_channel = "Nothing"]
#[group_prefix = "Prefix commands"]
async fn help_command( ctx: &Context
                     , msg: &Message
                     , args: Args
                     , help_options: &'static HelpOptions
                     , groups: &[&'static CommandGroup]
                     , owners: HashSet<UserId> ) -> CommandResult {
  if args.is_empty() {
    help_i18n(ctx, msg, &US_ENG).await;
  } else if help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await.is_none() {
    warn!("empty help answer");
  }
  Ok(())
}

#[instrument]
pub async fn run(opts : &IOptions) ->
  eyre::Result<(), Box<dyn std::error::Error + Send + Sync>> {
  { // this block limits scope of borrows by ap.refer() method
    let mut ap = ArgumentParser::new();
    let pname = "Amadeus";
    ap.set_description(pname);
    ap.add_option(&["--version"], Version(), "Show version");
    ap.parse_args_or_exit();
  }

  LogTracer::init()?;
  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;

  info!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());

  let http = serenity::http::Http::new_with_token(&opts.discord);

  info!("http context created");

  let (owners, amadeus_id) = match http.get_current_application_info().await {
    Ok(info) => {
      let mut owners = HashSet::new();
      if let Some(team) = info.team {
        owners.insert(team.owner_user_id);
      } else {
        owners.insert(info.owner.id);
      }
      (owners, info.id)
    },
    Err(why) => panic!("Could not access application info: {:?}", why),
  };

  info!("application info loaded");

  let runtime_options = options::get_roptions().await?;

  info!("all the options loaded");

  let mut creds = HashMap::new();
  creds.insert("tenor".to_string(), opts.tenor_key.clone());
  creds.insert("twitch_client".to_string(), opts.twitch_client_id.clone());
  creds.insert("twitch_secret".to_string(), opts.twitch_client_secret.clone());

  let mut core_guilds = HashMap::new();
  core_guilds.insert(CoreGuild::HEmo, opts.guild);
  core_guilds.insert(CoreGuild::Storage, opts.amadeus_guild);
  core_guilds.insert(CoreGuild::Amadeus, amadeus_id.0);

  let mut all_guilds = opts.servers.clone();
  all_guilds.push( IServer { id: opts.guild, kind: CoreGuild::HEmo } );
  all_guilds.push( IServer { id: opts.amadeus_guild, kind: CoreGuild::Storage } );

  let std_framework =
    StandardFramework::new()
     .configure(|c| c
      .owners(owners)
      .on_mention(Some(amadeus_id))
      .prefix("~")
      .delimiters(vec![" ", "\n"])
      .case_insensitivity(true))
      .on_dispatch_error(on_dispatch_error)
      .before(before)
      .after(after)
      .unrecognised_command(unrecognised_command)
      .group(&META_GROUP)
      .group(&CHAT_GROUP)
      .group(&TRANSLATE_GROUP)
      .group(&IMAGES_GROUP)
      .group(&WARCRAFT_GROUP)
      .group(&PAD_GROUP)
      .group(&LILYAL_GROUP)
      .group(&OWNER_GROUP)
      .group(&ADMIN_GROUP)
      .help(&HELP_COMMAND);

  let mut client =
    serenity::Client::builder(&opts.discord)
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
    data.insert::<AllGuilds>(Arc::new(all_guilds));
  }

  // start listening for events by starting a single shard
  if let Err(why) = client.start_autosharded().await {
    eprintln!("An error occurred while running the client: {:?}", why);
  }

  Ok(())
}

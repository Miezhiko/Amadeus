use crate::{
  common::{
    lang,
    msg::{ reply }
  },
  types::AOptions,
  handler::Handler,
  commands::{
    meta::*,
    voice::*,
    warcraft::*,
    owner::*,
    admin::*
  },
  commands::voice::VoiceManager,
  commands::meta::ShardManagerContainer,
  collections::base::{ GREETINGS, CONFUSION }
};

use serenity::{
  prelude::*,
  framework::StandardFramework,
  framework::standard::{
    DispatchError, Args, CommandOptions, CheckResult,
    macros::{group, check}
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

use markov::Chain;

use rand::{
  thread_rng,
  seq::SliceRandom
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
#[commands(ping, help)]
struct General;

#[group]
#[commands(join, leave, play)]
struct Voice;

#[group]
#[commands(lineup, yesterday, today, tomorrow, weekends)]
struct Warcraft;

#[group]
#[owners_only]
#[checks(Admin)]
#[commands(say)]
struct Owner;

#[group]
#[checks(Admin)]
#[commands(idle, stream)]
struct Admin;

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
  if let Some(member) = msg.member(&ctx.cache) {
    if let Ok(permissions) = member.permissions(&ctx.cache) {
      return permissions.administrator().into();
    }
  }
  false.into()
}

pub fn run(opts : &mut AOptions) -> Result<(), serenity::Error> {
  { // this block limits scope of borrows by ap.refer() method
    let mut ap = ArgumentParser::new();
    let pname = "Amadeus";
    ap.set_description(pname);
    ap.add_option(&["--version"], Version(), "Show version");
    ap.parse_args_or_exit();
  }

  let env = Env::default()
    .filter_or("MY_LOG_LEVEL", "info") // trace
    .write_style_or("MY_LOG_STYLE", "always");

  env_logger::init_from_env(env);

  info!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());

  let mut client = serenity::Client::new
    (&opts.discord, Handler).expect("Error creating serenity client");

  {
    let mut data = client.data.write();
    data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
  }

  let owners = match client.cache_and_http.http.get_current_application_info() {
    Ok(info) => {
      let mut set = HashSet::new();
      set.insert(info.owner.id);
      set
    },
    Err(why) => panic!("Couldn't get application info: {:?}", why),
  };

  let bot_id = match client.cache_and_http.http.get_current_application_info() {
    Ok(info) => {
      info.id
    },
    Err(why) => panic!("Could not access application info: {:?}", why),
  };

  client.with_framework(StandardFramework::new()
    .configure(|c| c
      .owners(owners)
      .on_mention(Some(bot_id))
      .prefix("~")
      .delimiters(vec![", ", ","])
      .case_insensitivity(true))

      .on_dispatch_error(|ctx, msg, error| {
        if let DispatchError::Ratelimited(seconds) = error {
          let _ = msg.channel_id.say(&ctx.http, &format!("Try this again in {} seconds.", seconds));
        }
      })

      .after(|_ctx, _msg, cmd_name, error| {
        match error {
          Ok(()) => trace!("Processed command '{}'", cmd_name),
          Err(why) => error!("Command '{}' returned error {:?}", cmd_name, why)
        }
      })

      .unrecognised_command(|ctx, msg, _unknown_command_name| {
        if let Some(_) = GREETINGS.into_iter().find(|&c| {
          let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
          let is_greeting = Regex::new(regex.as_str()).unwrap();
          is_greeting.is_match(msg.content.as_str()) }) 
        {
          let mut rng = thread_rng();
          set! { hi_reply = GREETINGS.choose(&mut rng).unwrap()
               , reply = format!("{}", hi_reply) };
          if let Err(why) = msg.reply(&ctx, reply) {
            error!("Error sending greeting reply: {:?}", why);
          }
        } else {
          if let Some(guild) = msg.guild(&ctx) {
            let msg_content = &msg.content;
            let russian = lang::is_russian(msg_content);
            let guild_id = guild.read().id;
            if let Ok(channels) = guild_id.channels(&ctx) {
              let main_channel = channels.iter().find(|&(c, _)|
                if let Some(name) = c.name(&ctx) {
                  name == "main"
                } else {
                  false
                });
              if let Some((_, _channel)) = main_channel {
                let mut chain = Chain::new();
                if let Ok(messages) = msg.channel_id.messages(&ctx, |r|
                  r.limit(6666)
                ) {
                  for mmm in messages {
                    let msg_content2 = &mmm.content;
                    if !msg_content2.contains("$") {
                      let is_russian = lang::is_russian(msg_content2.as_str());
                      if (russian && is_russian)
                      || (!russian && !is_russian) {
                        chain.feed_str(msg_content2.as_str());
                      }
                    }
                  }
                }
                if !russian {
                  for conf in CONFUSION {
                    chain.feed_str( conf );
                  }
                }
                chain.feed_str(msg_content.as_str());
                let mut answer = chain.generate_str();
                while answer.contains("@") || answer.contains("$") {
                  answer = chain.generate_str();
                }
                if !answer.is_empty() {
                  reply(&ctx, &msg, answer.as_str());
                }
              }
            }
          }
        }
      })

    .group(&GENERAL_GROUP)
    .group(&VOICE_GROUP)
    .group(&WARCRAFT_GROUP)
    .group(&OWNER_GROUP)
    .group(&ADMIN_GROUP)
  );

  client.start()
}

#[cfg(test)]
mod tests {
  // TODO: write tests
}

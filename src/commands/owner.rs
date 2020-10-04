use crate::{
  types::common::PubCreds,
  common::{ options, msg::{
      channel_message, direct_message
    }
  },
  steins::{ gate
          , ai::chain::{ ACTIVITY_LEVEL
                       , actualize_cache
                       , clear_cache }
          , ai::utils::capital_first },
};

use serenity::{
  model::{ id::ChannelId
         , gateway::Activity
         , channel::* },
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use serde_json::Value;

use std::sync::atomic::Ordering;

use regex::Regex;

use tokio::process::Command;

#[command]
#[min_args(2)]
#[owners_only]
async fn set(ctx: &Context, msg: &Message, mut args : Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  if let Ok(property) = args.single::<String>() {
    #[allow(clippy::single_match)]
    match property.as_str() {
      "activity" =>
        if let Ok(level) = args.single::<u32>() {
          ACTIVITY_LEVEL.store(level, Ordering::Relaxed);
          let chan_msg = format!("Activity level is: {} now", level);
          channel_message(&ctx, &msg, &chan_msg).await;
        },
      _ => ()
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[owners_only]
async fn say(ctx: &Context, msg: &Message, args : Args) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let last_channel_u64 = gate::LAST_CHANNEL.load(Ordering::Relaxed);
  if last_channel_u64 != 0 {
    let last_channel_conf = ChannelId( last_channel_u64 );
    if msg.guild_id.is_some() {
      let text = args.message();
      if !text.is_empty() {
        if let Err(why) = last_channel_conf.say(ctx, text).await {
          error!("Failed say {:?}", why);
        }
      }
    }
  }
  Ok(())
}

#[command]
#[owners_only]
async fn clear(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  if args.len() == 1 {
    let countdown: u64 = args.find().unwrap_or_default();
    if let Ok(vec) = msg.channel_id.messages(ctx, |g| g.before(msg.id).limit(countdown)).await {
      let mut vec_id = Vec::new();
      for message in vec {
        vec_id.push(message.id);
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, &msg, &format!("Deleted {} messages", countdown)).await;
  } else if args.len() == 2 {
    let countdown: usize = args.find().unwrap_or_default();
    let counter: usize = args.find().unwrap_or_default();
    let full = countdown + counter;
    if let Ok(vec) = msg.channel_id.messages(ctx, |g| g.before(msg.id).limit(full as u64)).await {
      let mut vec_id = Vec::new();
      for (i, message) in vec.iter().rev().enumerate() {
        if i < countdown {
          vec_id.push(message.id);
        }
      }
      vec_id.push(msg.id);
      match msg.channel_id.delete_messages(ctx, vec_id.as_slice()).await {
        Ok(val)  => val,
        Err(_err) => (),
      };
    }
    direct_message(ctx, &msg, &format!("Deleted {} messages", countdown)).await;
  }
  Ok(())
}

#[command]
#[owners_only]
async fn clear_chain_cache(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  clear_cache().await;
  channel_message(&ctx, &msg, "Cache removed").await;
  Ok(())
}

#[command]
#[owners_only]
async fn update_cache(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  actualize_cache(ctx).await;
  channel_message(ctx, msg, "Cache updated").await;
  Ok(())
}

#[command]
#[owners_only]
async fn upgrade(ctx: &Context, msg: &Message) -> CommandResult {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  ctx.set_activity(Activity::listening("Fetching changes")).await;
  ctx.idle().await;
  let git_fetch = Command::new("sh")
                  .arg("-c")
                  .arg("git fetch origin mawa")
                  .output()
                  .await
                  .expect("failed to execute git fetch");
  let git_reset = Command::new("sh")
                  .arg("-c")
                  .arg("git reset --hard origin/mawa")
                  .output()
                  .await
                  .expect("failed to reset on remote branch");
  if let Ok(git_fetch_out) = &String::from_utf8(git_fetch.stdout) {
    if let Ok(git_reset_out) = &String::from_utf8(git_reset.stdout) {
      let mut description = format!("{}\n{}", git_fetch_out, git_reset_out);
      let footer = format!("Requested by {}", msg.author.name);
      let mut mmm = msg.channel_id.send_message(&ctx, |m|
        m.embed(|e| e.title("Updating")
                     .colour((220, 20, 100))
                     .description(&description)
                     .footer(|f| f.text(&footer))
        )
      ).await?;
      ctx.set_activity(Activity::playing("Compiling...")).await;
      let cargo_update = Command::new("sh")
                .arg("-c")
                .arg("cargo update")
                .output()
                .await
                .expect("failed to update crates");
      let links_re = Regex::new(r"(.https.*)").unwrap();
      if let Ok(cargo_update_out) = &String::from_utf8(cargo_update.stderr) {
        let updating_git_re = Regex::new(r"(.Updating git.*)").unwrap();
        let mut update_str = links_re.replace_all(&cargo_update_out, "").to_string();
        update_str = updating_git_re.replace_all(&update_str, "").to_string();
        update_str = update_str.replace("/root/contrib/rust/", "");
        update_str = update_str.lines()
                               .filter(|l| !l.trim().is_empty())
                               .collect::<Vec<&str>>()
                               .join("\n");
        if update_str.len() > 666 {
          if let Some((i, _)) = update_str.char_indices().rev().nth(666) {
            update_str = update_str[i..].to_string();
          }
        }
        description = format!("{}\n{}", &description, update_str);
        mmm.edit(&ctx, |m|
          m.embed(|e| e.title("Compiling")
                       .colour((230, 10, 50))
                       .description(&description)
                       .footer(|f| f.text(&footer))
          )
        ).await?;
      }
      let cargo_build = Command::new("sh")
                .arg("-c")
                .arg("cargo build --release")
                .output()
                .await
                .expect("failed to compile new version");
      if let Ok(cargo_build_out) = &String::from_utf8(cargo_build.stderr) {
        let mut cut_paths = cargo_build_out.replace("/root/contrib/rust/", "");
        cut_paths = links_re.replace_all(&cut_paths, "").to_string();
        // if message is too big, take only last things
        if cut_paths.len() > 666 {
          if let Some((i, _)) = cut_paths.char_indices().rev().nth(666) {
            cut_paths = cut_paths[i..].to_string();
          }
        }
        description = format!("{}\n{}", &description, cut_paths);
        mmm.edit(&ctx, |m|
          m.embed(|e| e.title("Upgrading")
                       .colour((250, 0, 0))
                       .description(&description)
                       .footer(|f| f.text(&footer))
          )
        ).await?;
        ctx.set_activity(Activity::listening("Restarting")).await;
        let _systemctl = Command::new("sh")
                .arg("-c")
                .arg("systemctl restart Amadeus")
                .output()
                .await
                .expect("failed to restart Amadeus service");
        // I expect that we die right here
      }
    }
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

#[command]
#[owners_only]
async fn twitch_token_update(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  set!{ data            = ctx.data.read().await
      , client_id       = data.get::<PubCreds>().unwrap().get("twitch_client").unwrap().as_str()
      , client_secret   = data.get::<PubCreds>().unwrap().get("twitch_secret").unwrap().as_str() };
  let curl_command = format!(
    "curl -X POST \"https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials\"",
      client_id, client_secret);
  //TODO recode to simple request
  let curl = Command::new("sh")
    .arg("-c")
    .arg(&curl_command)
    .output()
    .await
    .expect("failed to run curl");
  if let Ok(curl_out) = &String::from_utf8(curl.stdout) {
    let json : Value = serde_json::from_str(&curl_out)?;
    if let Some(token_type) = json.pointer("/token_type") {
      let mut out = capital_first(token_type.as_str().unwrap());
      if let Some(access_token) = json.pointer("/access_token") {
        out = format!("{} {}", out, access_token.as_str().unwrap());
        let mut opts = options::get_roptions().await?;
        opts.twitch = out;
        options::put_roptions(&opts).await?;
        channel_message(&ctx, &msg, "twitch access token updated").await;
      }
    }
  }
  Ok(())
}

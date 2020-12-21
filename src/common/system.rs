use crate::{
  types::common::PubCreds,
  common::options,
  steins::gate::behavior::START_TIME
};

use serenity::{
  client::{
    bridge::gateway::{ShardId, ShardManager},
    bridge::voice::ClientVoiceManager
  },
  prelude::*,
  model::{ gateway::Activity
         , id::ChannelId }
};

use std::sync::Arc;

use chrono::{ Duration, Utc };
use tokio::process::Command;

use regex::Regex;
use once_cell::sync::Lazy;
use kathoey::utils::capital_first;
use serde_json::Value;

#[derive(Default, Debug)]
pub struct SysInfo {
  pub shard_latency: String,
  pub memory: String,
  pub db_size: String
}

pub struct VoiceManager;
pub struct ShardManagerContainer;

impl TypeMapKey for VoiceManager {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for ShardManagerContainer {
  type Value = Arc<Mutex<ShardManager>>;
}

pub async fn get_memory_mb() -> eyre::Result<f32> {
  let pid = std::process::id().to_string();
  let mem_stdout = Command::new("sh")
          .arg("-c")
          .arg(&format!("pmap {} | tail -n 1 | awk '/[0-9]K/{{print $2}}'", &pid))
          .output()
          .await?;
  let mem_used = &String::from_utf8(mem_stdout.stdout)?;
  Ok(mem_used[..mem_used.len() - 2].parse::<f32>().unwrap()/1024f32)
}

pub async fn get_system_info(ctx: &Context) -> SysInfo {
  let data = ctx.data.read().await;
  let mut sys_info = SysInfo::default();
  sys_info.shard_latency = {
    set! { shard_manager = data.get::<ShardManagerContainer>().unwrap()
         , manager       = shard_manager.lock().await
         , runners       = manager.runners.lock().await
         , runner_raw    = runners.get(&ShardId(ctx.shard_id)) };
    match runner_raw {
      Some(runner) => {
        match runner.latency {
          Some(ms) => format!("{}ms", ms.as_millis()),
          None => "?ms".to_string()
        }
      },
      None => "?ms".to_string()
    }
  };
  if let Ok(memory_mb) = get_memory_mb().await {
    sys_info.memory = if memory_mb >= 1024.0 {
      let memory_gb = memory_mb / 1024f32;
      format!("{:.3} GB", memory_gb)
      } else { format!("{:.3} MB", memory_mb) };
  } else {
    error!("Failed to parse mem stdout");
  }
  let dbs_stdout = Command::new("sh")
          .arg("-c")
          .arg("du -s trees | cut -f 1")
          .output()
          .await
          .expect("failed to execute process");
  if let Ok(db_size_str) = &String::from_utf8(dbs_stdout.stdout) {
    if let Ok(db_kb) = db_size_str[..db_size_str.len() - 1].parse::<u32>() {
      sys_info.db_size = if db_kb >= 1024 {
        let db_mb = db_kb as f32 / 1024f32;
        format!("{:.3} MB", db_mb)
        } else { format!("{:.3} KB", db_kb) };
    } else {
      error!("Failed to parse: {}", db_size_str);
      sys_info.db_size = String::from("?");
    }
  } else {
    error!("Failed to parse du stdout");
  }
  sys_info
}

pub async fn get_uptime(start: &str) -> (String, String) {
  let nao = Utc::now();
  let start_time = START_TIME.lock().await;
  let since_start_time : Duration = nao - *start_time;
  let mut uptime_string = String::from(start);

  let dd = since_start_time.num_days();
  if dd > 0 {
    uptime_string = format!("{} {}d", uptime_string, dd);
  }
  let hh = since_start_time.num_hours() - dd*24;
  if hh > 0 {
    uptime_string = format!("{} {}h", uptime_string, hh);
    if dd == 0 {
      let mm = since_start_time.num_minutes() - hh*60;
      uptime_string = format!("{} {}m", uptime_string, mm);
    }
  } else {
    let mm = since_start_time.num_minutes();
    if mm > 0 {
      uptime_string = format!("{} {}m {}s", uptime_string, mm
                                          , since_start_time.num_seconds() - mm*60);
    } else {
      uptime_string = format!("{} {}s", uptime_string, since_start_time.num_seconds());
    }
  }

  ( start_time.format("%Y %b %d %H:%M").to_string(), uptime_string )
}

pub async fn upgrade_amadeus(ctx: &Context, channel_id: &ChannelId) -> eyre::Result<()> {
  let start_typing = ctx.http.start_typing(channel_id.0);
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
      let mut mmm = channel_id.send_message(&ctx, |m|
        m.embed(|e| e.title("Updating")
                     .colour((220, 20, 100))
                     .description(&description)
        )
      ).await?;
      ctx.set_activity(Activity::playing("Compiling...")).await;
      let cargo_update = Command::new("sh")
                .arg("-c")
                .arg("cargo update")
                .output()
                .await
                .expect("failed to update crates");

      static LINKS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.https.*)").unwrap());
      if let Ok(cargo_update_out) = &String::from_utf8(cargo_update.stderr) {
        static GIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.Updating git.*)").unwrap());
        let mut update_str = LINKS_RE.replace_all(&cargo_update_out, "").to_string();
        update_str = GIT_RE.replace_all(&update_str, "").to_string();
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
          )
        ).await?;
      }
      let cargo_build = Command::new("sh")
                .arg("-c")
                .arg("cargo build --release --features flo")
                .output()
                .await
                .expect("failed to compile new version");
      if let Ok(cargo_build_out) = &String::from_utf8(cargo_build.stderr) {
        let mut cut_paths = cargo_build_out.replace("/root/contrib/rust/", "");
        cut_paths = LINKS_RE.replace_all(&cut_paths, "").to_string();
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

pub async fn twitch_update(ctx: &Context) -> eyre::Result<()> {
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
        return Ok(());
      }
    }
  }
  Err(eyre!("Failed to update twitch token"))
}

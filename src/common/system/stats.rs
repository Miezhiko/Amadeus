use crate::{
  common::system::ShardManagerContainer,
  steins::gate::START_TIME
};

use serenity::{
  client::bridge::gateway::ShardId,
  prelude::*
};

use chrono::{ Duration, Utc };
use tokio::process::Command;

#[derive(Default, Debug)]
pub struct SysInfo {
  pub shard_latency: String,
  pub memory: String,
  pub db_size: String
}

pub async fn get_memory_mb() -> anyhow::Result<f32> {
  let pid = std::process::id().to_string();
  let mem_stdout = Command::new("sh")
          .arg("-c")
          .arg(&format!("pmap {} | tail -n 1 | awk '/[0-9]K/{{print $2}}'", &pid))
          .output()
          .await?;
  let mem_used = &String::from_utf8(mem_stdout.stdout)?;
  Ok(mem_used[..mem_used.len() - 2].parse::<f32>().unwrap_or(0f32)/1024f32)
}

pub async fn get_system_info(ctx: &Context) -> SysInfo {
  let data = ctx.data.read().await;
  let mut sys_info = SysInfo {
    shard_latency: {
      set!{ shard_manager = data.get::<ShardManagerContainer>().unwrap()
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
    },
    ..Default::default()
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
  let since_start_time: Duration = nao - *start_time;
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
    let mm = since_start_time.num_minutes() - dd*24*60;
    if dd == 0 {
      if mm > 0 {
        uptime_string = format!("{} {}m {}s", uptime_string, mm
                                            , since_start_time.num_seconds() - mm*60);
      } else {
        uptime_string = format!("{} {}s", uptime_string, since_start_time.num_seconds());
      }
    } else {
      if mm > 0 {
        uptime_string = format!("{} {}m", uptime_string, mm);
      } else {
        uptime_string = format!("{} {}s", uptime_string, since_start_time.num_seconds() - dd*24*60*60);
      }
    }
  }

  ( start_time.format("%Y %b %d %H:%M").to_string(), uptime_string )
}

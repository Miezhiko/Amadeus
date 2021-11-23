use crate::{
  types::serenity::ReqwestClient,
  common::constants::GITHUB_PRS
};

use serenity::prelude::*;

use std::{ time, sync::Arc };

use tokio::process::Command;

use serde_json::Value;

/* every two minutes (maybe every minute is too much) */
static POLL_PERIOD_SECONDS: u64 = 2 * 60;

async fn parse_notification(ctx: &Context, rqcl: &reqwest::Client, json_str: &str) -> anyhow::Result<()> {
  let json_array = serde_json::from_str::<Value>(json_str)?;
  if let Some(array) = json_array.as_array() {
    for json in array {
      if let Some(subject) = json.pointer("/subject") {
        if let Some(last_read_at) = json.pointer("/last_read_at") {
          if last_read_at.is_null() {
            let title = subject.pointer("/title").unwrap_or(&Value::Null).as_str().unwrap_or_default();
            let url = subject.pointer("/url").unwrap_or(&Value::Null).as_str().unwrap_or_default();
            let res = rqcl.get(url).send().await?;
            let j = res.json::<Value>().await?;
            let state = j.pointer("/state").unwrap_or(&Value::Null).as_str().unwrap_or_default();
            if state != "closed" {
              let body = j.pointer("/body").unwrap_or(&Value::Null).as_str().unwrap_or_default();
              let html_url = j.pointer("/html_url").unwrap_or(&Value::Null).as_str().unwrap_or_default();
              if let Some(repo) = json.pointer("/repository") {
                let repository = repo.pointer("/full_name").unwrap_or(&Value::Null).as_str().unwrap_or_default();
                let avi =
                  if let Some(owner) = repo.pointer("/owner") {
                    owner.pointer("/avatar_url").unwrap_or(&Value::Null).as_str().unwrap_or_default()
                  } else { "https://cdn-icons-png.flaticon.com/512/25/25231.png" };
                let author =
                  if let Some(u) = json.pointer("/user") {
                    u.pointer("/login").unwrap_or(&Value::Null).as_str().unwrap_or_default()
                  } else { "" };
                GITHUB_PRS.send_message(ctx, |m| m
                  .embed(|e| { let mut e = e.title(&title)
                                            .author(|a| a.icon_url(avi).name(&repository))
                                            .url(&html_url);
                    if !body.is_empty() {
                      e = e.description(body);
                    }
                    if !author.is_empty() {
                      e = e.footer(|f| f.text(&format!("author: {}", author)));
                    }
                    e
                  })
                ).await?;
              }
            }
          }
        }
      }
    }
  }
  Ok(())
}

pub async fn activate_dev_tracker( ctx: &Arc<Context>
                                 , github_auth: &str ) {
  let ctx_clone = Arc::clone(ctx);
  let github: String = github_auth.to_string();
  tokio::spawn(async move {
    let rqcl = {
      set!{ data = ctx_clone.data.read().await
          , rqcl = data.get::<ReqwestClient>().unwrap() };
      rqcl.clone()
    };
    loop {
      {
        let fetch_command = format!("curl -u {} https://api.github.com/notifications?unread=true", &github);
        let curl = Command::new("sh")
          .arg("-c")
          .arg(&fetch_command)
          .output()
          .await
          .expect("failed to run dev GET curl");
        if let Ok(curl_out) = &String::from_utf8(curl.stdout) {
          if !curl_out.is_empty() {
            if let Err(why) = parse_notification(&ctx_clone, &rqcl, &curl_out).await {
              error!("Failed to parse github notifications {}", why);
            }
          }
        }
        // TODO: do this only if there was something really
        let put_command = format!("curl -u {} -X PUT -H \"Accept: application/vnd.github.v3+json\" https://api.github.com/notifications -d '{{\"read\":true}}'", &github);
        let _curl_put = Command::new("sh")
          .arg("-c")
          .arg(&put_command)
          .output()
          .await
          .expect("failed to run dev POST curl");
      }
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
    }
  });
}

use crate::common::constants::GITHUB_PRS;

use serenity::prelude::*;

use std::{ time, sync::Arc };

use tokio::process::Command;

use serde_json::Value;

use rand::Rng;

/* every 15 minutes */
static POLL_PERIOD_SECONDS: u64 = 15 * 60;

async fn parse_notification(ctx: &Context, github: &str, json_str: &str) -> anyhow::Result<bool> {
  let mut was_something = true;
  let json_array = serde_json::from_str::<Value>(json_str)?;
  if let Some(array) = json_array.as_array() {
    if array.is_empty() {
      was_something = false;
    }
    for json in array {
      if let Some(subject) = json.pointer("/subject") {
        if let Some(last_read_at) = json.pointer("/last_read_at") {
          if last_read_at.is_null() {
            set!{ title = subject.pointer("/title").unwrap_or(&Value::Null).as_str().unwrap_or_default()
                , url   = subject.pointer("/url").unwrap_or(&Value::Null).as_str().unwrap_or_default() };
            let pr_curl = format!("curl -u {} -H \"Accept: application/vnd.github.v3+json\" {}", github, url);
            let curl_pr = Command::new("sh")
              .arg("-c")
              .arg(&pr_curl)
              .output()
              .await?;
            if let Ok(curl_pr_out) = &String::from_utf8(curl_pr.stdout) {
              set!{ j     = serde_json::from_str::<Value>(curl_pr_out)?
                  , state = j.pointer("/state").unwrap_or(&Value::Null).as_str().unwrap_or_default() };
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
                    if let Some(u) = j.pointer("/user") {
                      ( u.pointer("/login").unwrap_or(&Value::Null).as_str().unwrap_or_default()
                      , u.pointer("/avatar_url").unwrap_or(&Value::Null).as_str().unwrap_or_default() )
                    } else { ( "", "https://cdn-icons-png.flaticon.com/512/25/25231.png" ) };
                  set!{ red   = rand::thread_rng().gen_range(0..255)
                      , green = rand::thread_rng().gen_range(0..255)
                      , blue  = rand::thread_rng().gen_range(0..255) };
                  GITHUB_PRS.send_message(ctx, |m| m
                    .embed(|e| { let mut e = e.title(title)
                                              .thumbnail(author.1)
                                              .author(|a| a.icon_url(avi).name(repository))
                                              .colour((red, green, blue))
                                              .url(html_url);
                      if !body.is_empty() {
                        e = e.description(body);
                      }
                      if !author.0.is_empty() {
                        e = e.footer(|f| f.text(&format!("author: {}", author.0)));
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
  }
  Ok(was_something)
}

pub async fn activate_dev_tracker( ctx: &Arc<Context>
                                 , github_auth: &str ) {
  let ctx_clone = Arc::clone(ctx);
  let github: String = github_auth.to_string();
  tokio::spawn(async move {
    loop {
      {
        let fetch_command = format!("curl -u {} https://api.github.com/notifications?unread=true", &github);
        let curl = Command::new("sh")
          .arg("-c")
          .arg(&fetch_command)
          .output()
          .await
          .expect("failed to run dev GET curl");
        let mut was_something = true;
        if let Ok(curl_out) = &String::from_utf8(curl.stdout) {
          if !curl_out.is_empty() {
            match parse_notification(&ctx_clone, &github, curl_out).await {
              Ok(r)     => was_something = r,
              Err(why)  => error!("Failed to parse github notifications {}", why)
            };
          }
        }
        if was_something {
          let put_command = format!("curl -u {} -X PUT -H \"Accept: application/vnd.github.v3+json\" https://api.github.com/notifications -d '{{\"read\":true}}'", &github);
          let _curl_put = Command::new("sh")
            .arg("-c")
            .arg(&put_command)
            .output()
            .await
            .expect("failed to run dev POST curl");
        }
      }
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
    }
  });
}

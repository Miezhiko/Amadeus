use crate::common::constants::GITHUB_PRS;

use serenity::prelude::*;

use std::{ time, sync::Arc };

use tokio::process::Command;

use serde_json::Value;

/* every minute */
static POLL_PERIOD_SECONDS: u64 = 1* 60;

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
        if let Ok(curl_out) = &String::from_utf8(curl.stdout) {
          if let Ok(json_array) = serde_json::from_str::<Value>(curl_out) {
            if let Some(array) = json_array.as_array() {
              for json in array {
                if let Some(subject) = json.pointer("/subject") {
                  if let Some(last_read_at) = json.pointer("/last_read_at") {
                    if last_read_at.is_null() {
                      let title = subject.pointer("/title").unwrap().as_str().unwrap();
                      let url = subject.pointer("/url").unwrap().as_str().unwrap();
                      if let Some(repo) = json.pointer("/repository") {
                        let repository = repo.pointer("/full_name").unwrap().as_str().unwrap();
                        let avi =
                          if let Some(owner) = json.pointer("/owner") {
                            owner.pointer("/avatar_url").unwrap().as_str().unwrap()
                          } else {
                            "https://cdn-icons-png.flaticon.com/512/25/25231.png"
                          };
                        if let Err(why) =
                          GITHUB_PRS.send_message(&ctx_clone, |m| m
                            .embed(|e| e.title(&title)
                                        .author(|a| a.icon_url(avi).name(&repository))
                                        .url(&url)
                          )).await {
                          error!("failed to post PR notification: {}", why);
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
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

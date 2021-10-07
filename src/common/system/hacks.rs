use crate::{
  types::serenity::PubCreds,
  common::options
};

use serenity::{
  prelude::*,
};

use tokio::process::Command;

use kathoey::utils::capital_first;
use serde_json::Value;

pub async fn twitch_update(ctx: &Context) -> anyhow::Result<()> {
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
    let json: Value = serde_json::from_str(&curl_out)?;
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
  Err(anyhow!("Failed to update twitch token"))
}

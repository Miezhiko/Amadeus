use tokio::process::Command;
use serde_json::Value;

/*
subheader.replay_length_ms
metadata.map
slot_records[x].race_flag .status .player_id
player_records .id .name
reforged_player_records .id .name .clan
*/

#[cfg(feature="w3g_rs")]
fn analyze_rs(path: &str) -> jane_eyre::Result<String> {
  let p = w3grs::parse(String::from(path))?;
  Ok( String::from( p.metadata.map ) )
}

#[cfg(not(feature = "w3g_rs"))]
async fn analyze_js(path: &str) -> jane_eyre::Result<String> {
  let node_out = Command::new("sh")
        .arg("-c")
        .arg(&format!("node js/w3gjs_parse.js {}", path))
        .output()
        .await?;
  let npm_stdout = String::from_utf8(node_out.stdout)?;
  if npm_stdout.is_empty() {
    let npm_stderr = String::from_utf8(node_out.stderr)?;
    info!("npm error: {}", &npm_stderr);
  }
  Ok(npm_stdout)
}

#[cfg(not(feature = "w3g_rs"))]
fn prettify_analyze_js(j: &str) -> (String, Vec<(String, String)>) {
  let j_res = serde_json::from_str(&j);
  if j_res.is_ok() {
    let json : Value = j_res.unwrap();
    let mut out = String::new();
    let mut pls = vec![];
    if let Some(map) = json.pointer("/map") {
      if let Some(file) = map.pointer("/file") {
        out = format!("**map**: {}\n", file.as_str().unwrap());
      }
    }
    if let Some(players) = json.pointer("/players") {
      for playa in players.as_array().unwrap().iter() {
        let mut p = String::new();
        let mut s = String::new();
        if let Some(name) = playa.pointer("/name") {
          p = format!("{}", name.as_str().unwrap());
        }
        if let Some(race) = playa.pointer("/race") {
          s = format!("**race**: {}\n", race.as_str().unwrap());
        }
        if let Some(apm) = playa.pointer("/apm") {
          s = format!("{}**apm**: {}", s, apm.as_u64().unwrap());
        }
        if let Some(heroes) = playa.pointer("/heroes") {
          for hero in heroes.as_array().unwrap().iter() {
            if let Some(id) = hero.pointer("/id") {
              let her = id.as_str().unwrap().to_uppercase();
              s = format!("{}\n**{}**", s, &her[1..]);
            }
            if let Some(level) = hero.pointer("/level") {
              s = format!("{} level {}", s, level.as_u64().unwrap());
            }
          }
        }
        pls.push((p, s));
      }
    }
    if let Some(duration) = json.pointer("/duration") {
      let dhuman = duration.as_u64().unwrap() / 100000;
      out = format!("{}**duration**: {}min", out, dhuman);
    }
    return (out, pls);
  }
  ( j.to_string(), vec![] )
}

#[cfg(feature="w3g_rs")]
pub async fn analyze(path: &str)
    -> jane_eyre::Result<(String, Vec<(String, String)>)> {
  let replay_data = analyze_rs(path)?;
  Ok(replay_data)
}

#[cfg(not(feature = "w3g_rs"))]
pub async fn analyze(path: &str)
    -> jane_eyre::Result<(String, Vec<(String, String)>)> {
  let replay_data = analyze_js(path).await?;
  Ok(prettify_analyze_js(&replay_data))
}

#[cfg(test)]
mod cyber_w3g_tests {
  use super::*;
  #[ignore]
  #[test]
  #[cfg(feature="w3g_rs")]
  fn parse_replay_test() {
    assert!( analyze_rs("example.w3g").is_ok() );
  }
  #[ignore]
  #[tokio::test(basic_scheduler)]
  #[cfg(not(feature = "w3g_rs"))]
  async fn my_test() -> Result<(), String> {
    if let Ok(replay_data) = analyze_js("example.w3g").await {
      assert!(!replay_data.is_empty());
      // this is for debug:
      // print!("{}", replay_data);
      let (_p, ps) = prettify_analyze_js(&replay_data);
      assert_eq!(2, ps.len());
      Ok(())
    } else {
      Err(String::from("Failed to get node output"))
    }
  }
}

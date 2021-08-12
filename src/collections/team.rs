use crate::types::team::Player;

use once_cell::sync::Lazy;

static HEMOD: &str   = "dhall/team/hemo.dhall";
static EXAMPLED: &str   = "dhall/team/example.dhall";

pub static HEMO: Lazy<Vec<Player>>    = Lazy::new(|| dhall!(HEMOD));
pub static EXAMPLE: Lazy<Vec<Player>> = Lazy::new(|| dhall!(EXAMPLED));

/*
pub fn players() -> Vec<Player> {
  [&HEMO[..], &EXAMPLE[..]].concat()
}
*/
pub fn players() -> Vec<Player> {
  [&HEMO[..]].concat()
}

#[cfg(test)]
mod stuff_dhall_tests {
  use super::*;
  fn dhall_players(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<Vec<Player>>() {
      Ok(some) => {
        if some.is_empty() {
          Err(String::from("empty structure loaded"))
        } else {
          Ok(())
        }
      }, Err(de) => Err(format!("Failed to parse {:?}", de))
    }
  }
  #[test]
  fn hemo() -> Result<(), String> { dhall_players(HEMOD) }
  #[test]
  fn example() -> Result<(), String> { dhall_players(EXAMPLED) }
}

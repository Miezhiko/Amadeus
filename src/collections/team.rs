use crate::types::team::Player;

use once_cell::sync::Lazy;

/*
* Note all the members of those lists are agreed to share this imformation
* If someone wants to be excluded please feel free to contact author or do it with PR
* Don't be toxic
*/

static DIVISION1D: &str   = "dhall/team/division1.dhall";
static DIVISION2D: &str   = "dhall/team/division2.dhall";
static INTERESTINGD: &str = "dhall/team/interesting.dhall";

pub static DIVISION1: Lazy<Vec<Player>>   = Lazy::new(|| dhall!(DIVISION1D));
pub static DIVISION2: Lazy<Vec<Player>>   = Lazy::new(|| dhall!(DIVISION2D));
pub static INTERESTING: Lazy<Vec<Player>> = Lazy::new(|| dhall!(INTERESTINGD));

pub fn teammates() -> Vec<Player> {
  [&DIVISION1[..], &DIVISION2[..]].concat()
}

pub fn players() -> Vec<Player> {
  [&DIVISION1[..], &DIVISION2[..], &INTERESTING[..]].concat()
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
  fn div1() -> Result<(), String> { dhall_players(DIVISION1D) }
  #[test]
  fn div2() -> Result<(), String> { dhall_players(DIVISION2D) }
  #[test]
  fn interesting() -> Result<(), String> { dhall_players(INTERESTINGD) }
}

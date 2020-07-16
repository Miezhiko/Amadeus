use crate::common::types::Player;

/*
* Note all the members of those lists are agreed to share this imformation
* If someone wants to be excluded please feel free to contact author or do it with PR
* Don't be toxic
*/

static DIVISION1D: &'static str   = "dhall/team/division1.dhall";
static DIVISION2D: &'static str   = "dhall/team/division2.dhall";
static INTERESTINGD: &'static str = "dhall/team/interesting.dhall";

lazy_static! {
  pub static ref DIVISION1: Vec<Player>   = dhall!(DIVISION1D);
  pub static ref DIVISION2: Vec<Player>   = dhall!(DIVISION2D);
  pub static ref INTERESTING: Vec<Player> = dhall!(INTERESTINGD);
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
        if some.len() > 0 {
          Ok(())
        } else {
          Err(String::from("empty structure loaded"))
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

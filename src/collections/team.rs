use crate::common::types::Player;

/*
* Note all the members of those lists are agreed to share this imformation
* If someone wants to be excluded please feel free to contact author or do it with PR
* Don't be toxic
*/

lazy_static! {

pub static ref DIVISION1: Vec<Player>   = dhall!("dhall/team/division1.dhall");

pub static ref DIVISION2: Vec<Player>   = dhall!("dhall/team/division2.dhall");

pub static ref INTERESTING: Vec<Player> = dhall!("dhall/team/interesting.dhall");

}

pub fn players() -> Vec<Player> {
  [&DIVISION1[..], &DIVISION2[..], &INTERESTING[..]].concat()
}

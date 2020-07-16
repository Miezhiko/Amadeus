use crate::types::common::Reaction;

static GREETINGSD: &'static str       = "dhall/base/greetings.dhall";
static CONFUSION_RUD: &'static str    = "dhall/base/confusion_ru.dhall";
static CONFUSIOND: &'static str       = "dhall/base/confusion.dhall";
static OBFUSCATION_RUD: &'static str  = "dhall/base/obfuscation_ru.dhall";
static OBFUSCATIOND: &'static str     = "dhall/base/obfuscation.dhall";
static REACTIONSD: &'static str       = "dhall/base/reactions.dhall";

lazy_static! {
  pub static ref GREETINGS: Vec<String>      = dhall!(GREETINGSD);
  pub static ref CONFUSION_RU: Vec<String>   = dhall!(CONFUSION_RUD);
  pub static ref CONFUSION: Vec<String>      = dhall!(CONFUSIOND);
  pub static ref OBFUSCATION_RU: Vec<String> = dhall!(OBFUSCATION_RUD);
  pub static ref OBFUSCATION: Vec<String>    = dhall!(OBFUSCATIOND);
  pub static ref REACTIONS: Vec<Reaction>    = dhall!(REACTIONSD);
}

#[cfg(test)]
mod base_dhall_tests {
  use super::*;
  fn dhall_vec(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<Vec<String>>() {
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
  fn greetings() -> Result<(), String> { dhall_vec(GREETINGSD) }
  #[test]
  fn confus_ru() -> Result<(), String> { dhall_vec(CONFUSION_RUD) }
  #[test]
  fn confusion() -> Result<(), String> { dhall_vec(CONFUSIOND) }
  #[test]
  fn obfusc_ru() -> Result<(), String> { dhall_vec(OBFUSCATION_RUD) }
  #[test]
  fn obfuscati() -> Result<(), String> { dhall_vec(OBFUSCATIOND) }
  #[test]
  fn reactions() -> Result<(), String> {
    match serde_dhall::from_file(REACTIONSD).parse::<Vec<Reaction>>() {
      Ok(some) => {
        if some.len() > 0 {
          Ok(())
        } else {
          Err(String::from("empty structure loaded"))
        }
      }, Err(de) => Err(format!("Failed to parse {:?}", de))
    }
  }
}

use crate::types::serenity::Reaction;

use once_cell::sync::Lazy;

static GREETINGSD: &str      = relative!("dhall/base/greetings.dhall");
static CONFUSION_RUD: &str   = relative!("dhall/base/confusion_ru.dhall");
static CONFUSIOND: &str      = relative!("dhall/base/confusion.dhall");
static OBFUSCATION_RUD: &str = relative!("dhall/base/obfuscation_ru.dhall");
static OBFUSCATIOND: &str    = relative!("dhall/base/obfuscation.dhall");
static WHITELISTD: &str      = relative!("dhall/base/whitelist.dhall");
static CASELISTD: &str       = relative!("dhall/base/case.dhall");
static REACTIONSD: &str      = relative!("dhall/base/reactions.dhall");

pub static GREETINGS: Lazy<Vec<String>>      = Lazy::new(|| dhall!(GREETINGSD));
pub static CONFUSION_RU: Lazy<Vec<String>>   = Lazy::new(|| dhall!(CONFUSION_RUD));
pub static CONFUSION: Lazy<Vec<String>>      = Lazy::new(|| dhall!(CONFUSIOND));
pub static OBFUSCATION_RU: Lazy<Vec<String>> = Lazy::new(|| dhall!(OBFUSCATION_RUD));
pub static OBFUSCATION: Lazy<Vec<String>>    = Lazy::new(|| dhall!(OBFUSCATIOND));
pub static WHITELIST: Lazy<Vec<u64>>         = Lazy::new(|| dhall!(WHITELISTD));
pub static CASELIST: Lazy<Vec<u64>>          = Lazy::new(|| dhall!(CASELISTD));
pub static REACTIONS: Lazy<Vec<Reaction>>    = Lazy::new(|| dhall!(REACTIONSD));

#[cfg(test)]
mod base_dhall_tests {
  use super::*;
  fn dhall_vec(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<Vec<String>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded"))
        }
      }, Err(de) => Err(format!("Failed to parse {de}"))
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
  fn whitelist() -> Result<(), String> { 
    match serde_dhall::from_file(WHITELISTD).parse::<Vec<u64>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded for whitelist"))
        }
      }, Err(de) => Err(format!("Failed to parse {de}"))
    }
  }
  #[test]
  fn case() -> Result<(), String> {
    match serde_dhall::from_file(CASELISTD).parse::<Vec<u64>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded for case"))
        }
      }, Err(de) => Err(format!("Failed to parse {de}"))
    }
  }
  #[test]
  fn reactions() -> Result<(), String> {
    match serde_dhall::from_file(REACTIONSD).parse::<Vec<Reaction>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded for reactions"))
        }
      }, Err(de) => Err(format!("Failed to parse {de}"))
    }
  }
}

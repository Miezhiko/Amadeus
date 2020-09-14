use crate::types::common::LChannel;

/*
 * Channels where I can spam (well, chat actually)
 */
static AI_ALLOWEDD: &str = "dhall/channels/ai_allowed.dhall";
/*
 * Channels where I really spam (well, chat)
 */
static EXCEPTIONSD: &str = "dhall/channels/exceptions.dhall";
/*
 * Channels where I can learn
 */
static AI_LEARND: &str   = "dhall/channels/ai_learn.dhall";
/*
 * Veto channels
 */
static IGNOREDD: &str   = "dhall/channels/ignored.dhall";

lazy_static! {
  pub static ref AI_ALLOWED: Vec<String> = dhall!(AI_ALLOWEDD);
  pub static ref EXCEPTIONS: Vec<String> = dhall!(EXCEPTIONSD);
  pub static ref AI_LEARN: Vec<LChannel> = dhall!(AI_LEARND);
  pub static ref IGNORED: Vec<String>    = dhall!(IGNOREDD);
}

#[cfg(test)]
mod channels_dhall_tests {
  use super::*;
  fn dhall_vec(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<Vec<String>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded"))
        }
      }, Err(de) => Err(format!("Failed to parse {:?}", de))
    }
  }
  #[test]
  fn ai_allowed() -> Result<(), String> { dhall_vec(AI_ALLOWEDD) }
  #[test]
  fn exceptions() -> Result<(), String> { dhall_vec(EXCEPTIONSD) }
  #[test]
  fn ai_learn() -> Result<(), String> {
    match serde_dhall::from_file(AI_LEARND).parse::<Vec<LChannel>>() {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty structure loaded"))
        }
      }, Err(de) => Err(format!("Failed to parse AI_LEARN channels {:?}", de))
    }
  }
  #[test]
  fn ignored() -> Result<(), String> { dhall_vec(IGNOREDD) }
}

/*
 * Channels where I can spam (well, chat actually)
 */
static AI_ALLOWEDD: &'static str = "dhall/channels/ai_allowed.dhall";
/*
 * Channels where I can learn
 */
static AI_LEARND: &'static str   = "dhall/channels/ai_learn.dhall";

lazy_static! {
  pub static ref AI_ALLOWED: Vec<String> = dhall!(AI_ALLOWEDD);
  pub static ref AI_LEARN: Vec<String>   = dhall!(AI_LEARND);
}

#[cfg(test)]
mod channels_dhall_tests {
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
  fn ai_allowed() -> Result<(), String> { dhall_vec(AI_ALLOWEDD) }
  #[test]
  fn ai_learn() -> Result<(), String> { dhall_vec(AI_LEARND) }
}

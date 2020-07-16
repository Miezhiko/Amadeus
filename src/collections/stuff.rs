static OVERWATCHD: &'static str = "dhall/stuff/overwatch.dhall";
static OVERWATCH_REPLIESD: &'static str = "dhall/stuff/overwatch_replies.dhall";

lazy_static! {
  pub static ref OVERWATCH: Vec<String>         = dhall!(OVERWATCHD);
  pub static ref OVERWATCH_REPLIES: Vec<String> = dhall!(OVERWATCH_REPLIESD);
}

#[cfg(test)]
mod stuff_dhall_tests {
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
  fn overwatch() -> Result<(), String> { dhall_vec(OVERWATCHD) }
  #[test]
  fn overwatch_replies() -> Result<(), String> { dhall_vec(OVERWATCH_REPLIESD) }
}

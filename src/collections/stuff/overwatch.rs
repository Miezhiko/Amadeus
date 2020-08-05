static OVERWATCHD: & str = "dhall/stuff/overwatch.dhall";
static OVERWATCH_REPLIESD: & str = "dhall/stuff/overwatch_replies.dhall";

lazy_static! {
  pub static ref OVERWATCH: Vec<String>         = dhall!(OVERWATCHD);
  pub static ref OVERWATCH_REPLIES: Vec<String> = dhall!(OVERWATCH_REPLIESD);
}

#[cfg(test)]
mod overwatch_dhall_tests {
  use super::*;
  fn dhall_vec(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<Vec<String>>() {
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
  fn overwatch() -> Result<(), String> { dhall_vec(OVERWATCHD) }
  #[test]
  fn overwatch_replies() -> Result<(), String> { dhall_vec(OVERWATCH_REPLIESD) }
}

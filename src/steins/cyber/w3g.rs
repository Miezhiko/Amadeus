use flo_w3replay::{ReplayInfo, W3Replay};

#[allow(dead_code)]
pub fn inspect(path: &str) -> Option<ReplayInfo> {
  if let Ok((inspect, _)) = W3Replay::inspect(path) {
    Some(inspect)
  } else {
    None
  }
}

#[cfg(test)]
mod cyber_w3g_tests {
  use super::*;
  #[test]
  fn get_map_test() {
    if let Some(i) = inspect("example.w3g") {
      let gn = i.game.game_name;
      let map = i.game.game_settings.map_path;
      assert_eq!(gn.to_str().unwrap(), "mawa");
      assert_eq!(map.to_str().unwrap(), "Maps/frozenthrone/community/(2)concealedhill.w3x");
    }
  }
}

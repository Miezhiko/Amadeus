use ucd::Codepoint;

pub fn is_russian(str: &str) -> bool {
  str.chars()
    .any(|c| matches!(
      c.script(), Some(ucd::Script::Cyrillic)
    )
  )
}

#[cfg(test)]
mod lang_tests {
  use super::*;
  #[test]
  fn is_russian_test() {
    assert!(is_russian("Тест"));
    assert!(!is_russian("Test"));
    assert!(is_russian("Тест Test"));
  }
}

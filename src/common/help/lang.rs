use ucd::Codepoint;

pub fn is_russian(str: &str) -> bool {
  str.chars()
    .any(|c| matches!(
      c.script(), Some(ucd::Script::Cyrillic)
    )
  )
}

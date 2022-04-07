use crate::{
  types::rules::*,
  collections::stuff::owo::*
};

use kathoey::utils::capital_first;

fn spell_word(word: &str) -> String {
  if word.is_empty() { return String::new(); }

  setm! { whole_word_uppercase  = true
        , first_char_uppercase  = true
        , first_char_checked    = false };

  for ch in word.chars() {
    if ch.is_lowercase() {
      if !first_char_checked {
        first_char_uppercase = false;
      }
      whole_word_uppercase = false;
      break;
    }
    first_char_checked = true;
  }

  let mut result = word.to_lowercase();

  for rule in RULES.iter() {
    result = match rule {
      Rule::Regex(re) => re.from.replace_all(&result, re.to).to_string(),
      Rule::Function(h) => (h.function)(&result),
    };
  }

  if whole_word_uppercase {
    result = result.to_uppercase();
  } else if first_char_uppercase {
    result = capital_first(&result);
  }

  result
}

pub fn spell(text: &str) -> String {
  if text.is_empty() { return String::new(); }
  let mut out = vec![];
  for word in text.split_whitespace() {
    let spelled = spell_word(word);
    if !spelled.is_empty() {
      out.push(spelled);
    }
  }
  out.join(" ")
}

#[cfg(test)]
mod owo_tests {
  use super::*;
  #[test]
  fn uwu_test() {
    assert_eq!(
      spell("Wanderbraun"),
        "Wandewbwaun"
    );
    assert_eq!(
      spell("Fingon fingoning"),
        "Fingon fingonying"
    );
  }
}

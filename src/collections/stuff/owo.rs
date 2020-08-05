use crate::types::rules::*;

use regex::Regex;

lazy_static! {
  pub static ref RULES: Vec<Rule> = vec![
    bjr! { r"(?:r|l)"     => "w" },
    bjr! { r"(?:R|L)"     => "W" },
    bjr! { r"n([aeiou])"  => "ny$1" },
    bjr! { r"N([aeiou])"  => "Ny$1" },
    bjr! { r"N([AEIOU])"  => "NY$1" },
    bjr! { r"th"          => "d" },
    bjr! { r"ove"         => "uv" }
  ];
}

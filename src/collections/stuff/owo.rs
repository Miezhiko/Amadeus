use crate::types::rules::*;

use regex::Regex;

use once_cell::sync::Lazy;

pub static RULES: Lazy<Vec<Rule>> = Lazy::new(||
  vec![
    bjr! { r"(?:r|l)"     => "w" },
    bjr! { r"(?:R|L)"     => "W" },
    bjr! { r"n([aeiou])"  => "ny$1" },
    bjr! { r"N([aeiou])"  => "Ny$1" },
    bjr! { r"N([AEIOU])"  => "NY$1" },
    bjr! { r"th"          => "d" },
    bjr! { r"ove"         => "uv" }
  ]
);

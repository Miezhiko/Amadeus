pub mod translation;

use regex::Regex;

use once_cell::sync::Lazy;

pub const LUKASHENKO: &str  = "lukashenko";

pub static GPT_LIMIT: usize = 1000;

pub static RE1: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
pub static RE2: Lazy<Regex> = Lazy::new(|| Regex::new(r":(.*?):").unwrap());
pub static RE3: Lazy<Regex> = Lazy::new(|| Regex::new(r"&(.*?);").unwrap());
pub static RE4: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

pub fn process_message_for_gpt(s: &str) -> String {
  let mut result_string = RE1.replace_all(s, "").to_string();
  result_string = RE2.replace_all(&result_string, "").to_string();
  result_string = RE3.replace_all(&result_string, "").to_string();
  result_string = RE4.replace_all(&result_string, " ").to_string();
  result_string.trim().to_string()
}

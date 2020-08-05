use regex::Regex;

#[derive(Clone, Debug)]
pub struct RuleRe {
  pub from: Regex,
  pub to: &'static str,
}

pub type RuleFnType = fn(&str) -> String;

#[derive(Clone)]
pub struct RuleFn {
  pub function: RuleFnType,
}

#[derive(Clone)]
pub enum Rule {
  Regex(RuleRe),
  Function(RuleFn),
}

impl Rule {
  pub fn new_re(from: Regex, to: &'static str) -> Self {
    Rule::Regex(RuleRe { from, to })
  }
  pub fn new_fn(function: RuleFnType) -> Self {
    Rule::Function(RuleFn { function })
  }
}

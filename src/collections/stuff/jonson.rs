use crate::types::rules::*;

use itertools::Itertools;
use regex::Regex;

fn remove_duplicate_characters(data: &str) -> String {
  data.chars().dedup().collect()
}

lazy_static! {
  pub static ref RULES: Vec<Rule> = vec![
    bjr! { r"[ъь]"    => "" },
    bjr! { r"[^а-я]"  => "" },
    bjf! { remove_duplicate_characters },
    bjr! { "йо|ио|йе|ие"  => "и" },
    bjr! { "[оыя]"        => "а" },
    bjr! { "[ейэ]"        => "и" },
    bjr! { "ю"            => "У" },
    bjr! { "б(б|в|г|д|ж|з|й|к|п|с|т|ф|х|ц|ч|ш|щ)" => "п$1" },
    bjr! { "б$" => "п" },
    bjr! { "з(б|в|г|д|ж|з|й|к|п|с|т|ф|х|ц|ч|ш|щ)" => "с$1" },
    bjr! { "з$" => "с" },
    bjr! { "д(б|в|г|д|ж|з|й|к|п|с|т|ф|х|ц|ч|ш|щ)" => "т$1" },
    bjr! { "д$" => "т" },
    bjr! { "в(б|в|г|д|ж|з|й|к|п|с|т|ф|х|ц|ч|ш|щ)" => "ф$1" },
    bjr! { "в$" => "ф" },
    bjr! { "г(б|в|г|д|ж|з|й|к|п|с|т|ф|х|ц|ч|ш|щ)" => "к$1" },
    bjr! { "г$"     => "к" },
    bjr! { "тс|дс"  => "ц" },
    bjf! { remove_duplicate_characters },
  ];
}

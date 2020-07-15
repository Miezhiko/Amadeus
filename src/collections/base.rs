lazy_static! {

pub static ref GREETINGS: Vec<String>        = dhall!("dhall/base/greetings.dhall");

pub static ref CONFUSION_RU: Vec<String>     = dhall!("dhall/base/confusion_ru.dhall");

pub static ref CONFUSION: Vec<String>        = dhall!("dhall/base/confusion.dhall");

pub static ref OBFUSCATION_RU: Vec<String>   = dhall!("dhall/base/obfuscation_ru.dhall");

pub static ref OBFUSCATION: Vec<String>      = dhall!("dhall/base/obfuscation.dhall");

pub static ref REACTIONS: Vec<(u64, String)> = dhall!("dhall/base/reactions.dhall");

}

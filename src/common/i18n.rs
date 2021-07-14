use serenity::{ prelude::*
              , model::channel::* };

use unic_langid::{ LanguageIdentifier, langid };
use fluent_templates::{ Loader, static_loader };

pub const US_ENG: LanguageIdentifier = langid!("en-US");
pub const RU: LanguageIdentifier = langid!("ru-RU");

static_loader! {
  static LOCALES = {
    locales: "./locales",
    fallback_language: "en-US"
  };
}

pub async fn help_i18n(ctx: &Context, msg: &Message, lang: &LanguageIdentifier) {
  let version = format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.channel_id.send_message(ctx, |m| m
    .embed(|e| e
      .title("Amadeus")
      .url("https://github.com/Miezhiko/Amadeus")
      .image("https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .description(LOCALES.lookup(lang, "help-description"))
      .fields(vec![
        (LOCALES.lookup(lang, "age"), "18", true),
        (LOCALES.lookup(lang, "birthdate"), &LOCALES.lookup(lang, "amadeus-birthdate"), true),
        (LOCALES.lookup(lang, "blood-type"), "A", true)
        ])
      .fields(vec![
        (LOCALES.lookup(lang, "height"), &LOCALES.lookup(lang, "amadeus-height"), true),
        (LOCALES.lookup(lang, "weight"), &LOCALES.lookup(lang, "amadeus-weight"), true),
        (LOCALES.lookup(lang, "version"), &version, true)
        ])
      .field(LOCALES.lookup(lang, "user-commands-title")
           , LOCALES.lookup(lang, "user-commands"), false)
      .field(LOCALES.lookup(lang, "music-commands-title")
           , LOCALES.lookup(lang, "music-commands"), false)
      .field(LOCALES.lookup(lang, "warcraft-commands-title")
           , LOCALES.lookup(lang, "warcraft-commands"), false)
      .footer(|f| f.text(LOCALES.lookup(lang, "footer")))
      .colour((246, 111, 0)))).await {
    error!("Error sending help message: {:?}", why);
  }
}

pub async fn edit_help_i18n(ctx: &Context, msg: &mut Message, lang: &LanguageIdentifier) {
  let version = format!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());
  if let Err(why) = msg.edit(ctx, |m| m
    .content("")
    .embed(|e| e
      .title("Amadeus")
      .url("https://github.com/Miezhiko/Amadeus")
      .image("https://vignette.wikia.nocookie.net/steins-gate/images/8/83/Kurisu_profile.png")
      .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
      .description(LOCALES.lookup(lang, "help-description"))
      .fields(vec![
        (LOCALES.lookup(lang, "age"), "18", true),
        (LOCALES.lookup(lang, "birthdate"), &LOCALES.lookup(lang, "amadeus-birthdate"), true),
        (LOCALES.lookup(lang, "blood-type"), "A", true)
        ])
      .fields(vec![
        (LOCALES.lookup(lang, "height"), &LOCALES.lookup(lang, "amadeus-height"), true),
        (LOCALES.lookup(lang, "weight"), &LOCALES.lookup(lang, "amadeus-weight"), true),
        (LOCALES.lookup(lang, "version"), &version, true)
        ])
      .field(LOCALES.lookup(lang, "user-commands-title")
           , LOCALES.lookup(lang, "user-commands"), false)
      .field(LOCALES.lookup(lang, "music-commands-title")
           , LOCALES.lookup(lang, "music-commands"), false)
      .field(LOCALES.lookup(lang, "warcraft-commands-title")
           , LOCALES.lookup(lang, "warcraft-commands"), false)
      .footer(|f| f.text(LOCALES.lookup(lang, "footer")))
      .colour((246, 111, 0)))).await {
    error!("Error editing help message: {:?}", why);
  }
}

/*
Embed titles are limited to 256 characters
Embed descriptions are limited to 2048 characters
There can be up to 25 fields
A field's name is limited to 256 characters and its value to 1024 characters
The footer text is limited to 2048 characters
The author name is limited to 256 characters
In addition, the sum of all characters in an embed structure must not exceed 6000 characters
*/
#[cfg(test)]
mod translation_tests {
  use super::*;
  fn help_test_with_lang(lang: &LanguageIdentifier) {
    let string_description = LOCALES.lookup(lang, "help-description");
    assert!( string_description.chars().count() < 2048 );
    let string_footer = LOCALES.lookup(lang, "footer");
    assert!( string_footer.chars().count() < 2048 );
    let strings = &[
      string_description,
      LOCALES.lookup(lang, "age"),
      LOCALES.lookup(lang, "birthdate"),
      LOCALES.lookup(lang, "amadeus-birthdate"),
      LOCALES.lookup(lang, "blood-type"),
      LOCALES.lookup(lang, "height"),
      LOCALES.lookup(lang, "amadeus-height"),
      LOCALES.lookup(lang, "weight"),
      LOCALES.lookup(lang, "amadeus-weight"),
      LOCALES.lookup(lang, "version"),
      LOCALES.lookup(lang, "user-commands-title"),
      LOCALES.lookup(lang, "user-commands"),
      LOCALES.lookup(lang, "music-commands-title"),
      LOCALES.lookup(lang, "music-commands"),
      LOCALES.lookup(lang, "warcraft-commands-title"),
      LOCALES.lookup(lang, "warcraft-commands"),
      string_footer
    ];
    assert!( strings.iter().map(|s| s.chars().count()).sum::<usize>() < 6000 );
  }
  #[test]
  fn help_test_english() {
    help_test_with_lang(&US_ENG);
  }
  #[test]
  fn help_test_russian() {
    help_test_with_lang(&RU);
  }
}

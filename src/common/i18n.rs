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
      .url("https://github.com/Qeenon/Amadeus")
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

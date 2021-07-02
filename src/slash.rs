use crate::{
  message::RESTORE,
  steins::{ ai::boris
          , ai::uwu
          },
  common::i18n::{ edit_help_i18n, US_ENG },
  commands::{
    translation, w3c::stats,
    chat
  }
};

use serenity::{
  prelude::*,
  model::{ guild::PartialGuild
         , interactions::{ ApplicationCommandOptionType
                         , InteractionResponseType
                         , Interaction
                         , InteractionData }
         },
  framework::standard::{ Args, Delimiter }
};

use std::sync::atomic::Ordering;

static ASYNC_CMDS: [&str; 6] = [ "translate"
                               , "перевод"
                               , "help"
                               , "stats"
                               , "феминизировать"
                               , "correct" ];

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.create_application_commands(ctx, |cs| {
    cs.create_application_command(|c| c.name("help")
      .description("Display Amadeus Help")
    )
      .create_application_command(|c| c.name("translate")
      .description("Translate Russian to English")
      .create_option(|o| {
          o.name("text")
          .description("What will be translated")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("перевод")
      .description("Перевод с английского на Русский")
      .create_option(|o| {
          o.name("текст")
          .description("Текст для перевода")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("stats")
      .description("Display W3C player statistics")
      .create_option(|o| {
          o.name("battletag")
          .description("Target player")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("борис")
      .description("Команда, которую любит Лилуал")
      .create_option(|o| {
          o.name("текст")
          .description("Текст для Бориса")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("uwu")
      .description("Uwufy some text OwO")
      .create_option(|o| {
          o.name("text")
          .description("Some text...")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("феминизировать")
      .description("Феминизировать предложение")
      .create_option(|o| {
          o.name("текст")
          .description("Текст для феминизации")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("correct")
      .description("Correct grammar for English text")
      .create_option(|o| {
          o.name("text")
          .description("Text for correction")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
  }).await {
    error!("Failed to register global application commands {:?}", why);
  }
}

pub async fn handle_slash_commands(ctx: &Context, interaction: &Interaction) {
  if let Some(d) = &interaction.data {
    match d {
      InteractionData::ApplicationCommand(ac) => {
        match ac.name.as_str() {
          "борис" => {
            if let Some(o) = ac.options.first() {
              if let Some(v) = o.value.clone() {
                if let Some(t) = v.as_str() {

                  if let Err(why) = interaction.create_interaction_response(&ctx.http, |response| {
                    response
                      .kind(InteractionResponseType::ChannelMessageWithSource)
                      .interaction_response_data( |message| message.content( boris::spell(t) ) )
                  }).await {
                    error!("Failed to create boris interaction response {:?}", why);
                  }

                }
              }
            }
          },
          "uwu" => {
            if let Some(o) = ac.options.first() {
              if let Some(v) = o.value.clone() {
                if let Some(t) = v.as_str() {

                  if let Err(why) = interaction.create_interaction_response(&ctx.http, |response| {
                    response
                      .kind(InteractionResponseType::ChannelMessageWithSource)
                      .interaction_response_data( |message| message.content( uwu::spell(t) ) )
                  }).await {
                    error!("Failed to create uwu interaction response {:?}", why);
                  }

                }
              }
            }
          },

          c if ASYNC_CMDS.iter().any(|cmd| c == *cmd) => {

            if let Err(why) = interaction.create_interaction_response(&ctx.http, |response| {
              response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data( |message| message.content("OK!") )
            }).await {
              error!("Failed to create OK interaction response {:?}", why);
            }

            match c {
              "help" => {
                match interaction.edit_original_interaction_response(&ctx.http, |response|
                  response.content("Creating Help ...")
                ).await {
                  Ok(mut msg) => {
                    edit_help_i18n(&ctx, &mut msg, &US_ENG).await;
                  }, Err(why) => {
                    error!("Failed to create help interaction response {:?}", why);
                  }
                };
              },
              "stats" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      RESTORE.store(false, Ordering::Relaxed);
                      match interaction.edit_original_interaction_response(&ctx.http, |response|
                        response.content(&format!("Getting stats for {}", t))
                      ).await {
                        Ok(msg) => {
                          let args = Args::new(t, &[Delimiter::Single(';')]);
                          if let Err(serr) = stats(&ctx, &msg, args).await {
                            error!("Failed to get stats on interaction {:?}", serr);
                          }
                        }, Err(why) => {
                          error!("Failed to create stats interaction response {:?}", why);
                        }
                      };
                      RESTORE.store(true, Ordering::Relaxed);

                    }
                  }
                }
              },
              "correct" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      RESTORE.store(false, Ordering::Relaxed);
                      match interaction.edit_original_interaction_response(&ctx.http, |response|
                        response.content(&format!("Correcting {}", t))
                      ).await {
                        Ok(msg) => {
                          let args = Args::new(t, &[Delimiter::Single(';')]);
                          if let Err(serr) = chat::correct(&ctx, &msg, args).await {
                            error!("Failed to correct on interaction {:?}", serr);
                          }
                        }, Err(why) => {
                          error!("Failed to correct on interaction response {:?}", why);
                        }
                      };
                      RESTORE.store(true, Ordering::Relaxed);

                    }
                  }
                }
              },
              "феминизировать" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      RESTORE.store(false, Ordering::Relaxed);
                      match interaction.edit_original_interaction_response(&ctx.http, |response|
                        response.content(&format!("Феминизация \"{}\"", t))
                      ).await {
                        Ok(msg) => {
                          let args = Args::new(t, &[Delimiter::Single(';')]);
                          if let Err(serr) = chat::feminize(&ctx, &msg, args).await {
                            error!("Failed to feminize on interaction {:?}", serr);
                          }
                        }, Err(why) => {
                          error!("Failed to feminize on interaction response {:?}", why);
                        }
                      };
                      RESTORE.store(true, Ordering::Relaxed);

                    }
                  }
                }
              },
              cmd if cmd == "translate" || cmd == "перевод" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      RESTORE.store(false, Ordering::Relaxed);
                      match interaction.edit_original_interaction_response(&ctx.http, |response|
                        response.content(&format!("Translating {}", t))
                      ).await {
                        Ok(msg) => {
                          let args = Args::new(t, &[Delimiter::Single(';')]);
                          if cmd == "translate" {
                            if let Err(terr) = translation::translate(&ctx, &msg, args).await {
                              error!("Failed to translate to English on interaction {:?}", terr);
                            }
                          } else {
                            if let Err(terr) = translation::perevod(&ctx, &msg, args).await {
                              error!("Failed to translate to Russian on interaction {:?}", terr);
                            }
                          }
                        }, Err(why) => {
                          error!("Failed to create translation interaction response {:?}", why);
                        }
                      };
                      RESTORE.store(true, Ordering::Relaxed);

                    }
                  }
                }
              }
              _ => { /* dunno */ }
            };
          },

          _ => { /* dunno */ }
        };
      },
      _ => { /* dunno */ }
    }
  }
}

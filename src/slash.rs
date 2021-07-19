use crate::{
  message::RESTORE,
  steins::{ ai::boris
          , ai::uwu
          },
  common::i18n::{ edit_help_i18n, US_ENG },
  commands::{
    translation, w3c::stats,
    chat, meta, music, images
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

static ASYNC_CMDS: [&str; 14] = [ "translate", "перевод", "help"
                                , "stats", "феминизировать", "correct"
                                , "time", "время", "leave"
                                , "play", "repeat"
                                , "wave", "cry", "hug" ];

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.create_application_commands(ctx, |cs| {
    cs.create_application_command(|c| c.name("help")
      .description("Display Amadeus Help")
    )
      .create_application_command(|c| c.name("wave")
      .description("Wave a hand you know...")
    )
      .create_application_command(|c| c.name("cry")
      .description("Start to cry!")
    )
      .create_application_command(|c| c.name("hug")
      .description("Literally hug someone")
      .create_option(|o| {
          o.name("person")
          .description("Person to hug")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
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
      .create_application_command(|c| c.name("time")
      .description("Display current time")
      .create_option(|o| {
          o.name("timezone")
          .description("Optional timezone")
          .kind(ApplicationCommandOptionType::String)
          .required(false)
      })
    )
      .create_application_command(|c| c.name("время")
      .description("Показать текущее время")
      .create_option(|o| {
          o.name("город")
          .description("Дополнительный часовой пояс")
          .kind(ApplicationCommandOptionType::String)
          .required(false)
      })
    )
      .create_application_command(|c| c.name("join")
      .description("Join voice channel with you (you should be in voice channel)")
    )
      .create_application_command(|c| c.name("leave")
      .description("Leave voice channel")
    )
      .create_application_command(|c| c.name("repeat")
      .description("Play last song again")
    )
      .create_application_command(|c| c.name("play")
      .description("Play radio stream or youtube stuff")
      .create_option(|o| {
          o.name("url")
          .description("link for music to play")
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
          "join" => {
            if let Some(guild_id) = &interaction.guild_id {
              if let Some(guild) = guild_id.to_guild_cached(ctx).await {
                if let Some(member) = &interaction.member {
                  if let Err(err) = music::join_slash(ctx, &member.user, &guild).await {
                    if let Err(why) = interaction.create_interaction_response(&ctx.http, |response| {
                      response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data( |message| message.content( format!("Failed to join {:?}", err) ) )
                    }).await {
                      error!("Failed to create boris interaction response {:?}", why);
                    }
                  } else {
                    if let Err(why) = interaction.create_interaction_response(&ctx.http, |response| {
                      response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data( |message| message.content( "I've joined voice channel" ) )
                    }).await {
                      error!("Failed to create boris interaction response {:?}", why);
                    }
                  }
                }
              }
            }
          }
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
                    edit_help_i18n(ctx, &mut msg, &US_ENG).await;
                  }, Err(why) => {
                    error!("Failed to create help interaction response {:?}", why);
                  }
                };
              },
              "wave" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Waving ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "wave anime"
                                    , 0x3252e3
                                    , images::own("waves")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "cry" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Crying ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "cry anime"
                                    , 0x126223
                                    , images::own("crying")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "hug" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      if let Some(member) = &interaction.member {
                        match interaction.edit_original_interaction_response(&ctx.http, |response|
                          response.content("Hugs ...")
                        ).await {
                          Ok(mut msg) => {
                            if let Err(err) =
                              images::gifs( ctx, &member.user
                                          , &mut msg
                                          , "hug anime"
                                          , 0xed9e2f
                                          , images::target("hugs")
                                          , false, Some(t.into())).await {
                              error!("Failed do gif emoji {:?}", err);
                            }
                          }, Err(why) => {
                            error!("Failed to create help interaction response {:?}", why);
                          }
                        };
                      }

                    }
                  }
                }
              },
              "leave" => {
                match interaction.edit_original_interaction_response(&ctx.http, |response|
                  response.content("Leaving...")
                ).await {
                  Ok(msg) => {
                    RESTORE.store(false, Ordering::Relaxed);
                    let args = Args::new("", &[]);
                    if let Err(err) = music::leave(ctx, &msg, args).await {
                      error!("Failed to leave voice channel {:?}" , err);
                    }
                    RESTORE.store(true, Ordering::Relaxed);
                  }, Err(why) => {
                    error!("Failed to create leave interaction response {:?}", why);
                  }
                };
              },
              "repeat" => {
                match interaction.edit_original_interaction_response(&ctx.http, |response|
                  response.content("Repeating...")
                ).await {
                  Ok(msg) => {
                    RESTORE.store(false, Ordering::Relaxed);
                    let args = Args::new("", &[]);
                    if let Err(err) = music::play(ctx, &msg, args).await {
                      error!("Failed to repeat last song {:?}" , err);
                    }
                    RESTORE.store(true, Ordering::Relaxed);
                  }, Err(why) => {
                    error!("Failed to create repeat interaction response {:?}", why);
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
              "play" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      RESTORE.store(false, Ordering::Relaxed);
                      match interaction.edit_original_interaction_response(&ctx.http, |response|
                        response.content(&format!("Playing {}", t))
                      ).await {
                        Ok(msg) => {
                          let args = Args::new(t, &[Delimiter::Single(';')]);
                          if let Err(serr) = music::play(&ctx, &msg, args).await {
                            error!("Failed to get play on interaction {:?}", serr);
                          }
                        }, Err(why) => {
                          error!("Failed to create play interaction response {:?}", why);
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
              },
              cmd if cmd == "time" || cmd == "время" => {
                let mut str_arg = String::new();
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {
                      str_arg = t.into();
                    }
                  }
                }
                RESTORE.store(false, Ordering::Relaxed);
                match interaction.edit_original_interaction_response(&ctx.http, |response|
                  response.content("⌚")
                ).await {
                  Ok(msg) => {
                    let args = Args::new(str_arg.as_str(), &[Delimiter::Single(';')]);
                    if let Err(terr) = meta::time(&ctx, &msg, args).await {
                      error!("Failed to show time on interaction {:?}", terr);
                    }
                  }, Err(why) => {
                    error!("Failed to show time interaction response {:?}", why);
                  }
                };
                RESTORE.store(true, Ordering::Relaxed);
              },
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

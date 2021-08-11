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

static ASYNC_CMDS: [&str; 52] = [ "translate", "перевод", "help"
                                , "stats", "феминизировать", "correct"
                                , "time", "время", "leave"
                                , "play", "repeat", "scared"
                                , "wave", "cry", "hug"
                                , "pat", "slap", "cringe"
                                , "ahegao", "clap", "wink"
                                , "shrug", "lol", "angry"
                                , "dance", "confused", "shock"
                                , "nervous", "sad", "happy"
                                , "annoyed", "omg", "smile"
                                , "ew", "awkward", "oops"
                                , "lazy", "hungry", "stressed"
                                , "scard", "bored", "yes", "no"
                                , "bye", "sorry", "sleepy"
                                , "facepalm", "whatever", "pout"
                                , "smug", "smirk", "gif" ];

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
      .create_application_command(|c| c.name("cringe")
      .description("just cringe")
    )
      .create_application_command(|c| c.name("ahegao")
      .description("Make an ahegao face")
    )
      .create_application_command(|c| c.name("clap")
      .description("Start clapping")
    )
      .create_application_command(|c| c.name("shrug")
      .description("Shrug shoulders")
    )
      .create_application_command(|c| c.name("lol")
      .description("laugh out loud")
    )
      .create_application_command(|c| c.name("angry")
      .description("Angry feels")
    )
      .create_application_command(|c| c.name("dance")
      .description("Dance Dance Dance")
    )
      .create_application_command(|c| c.name("confused")
      .description("Shows your confusion")
    )
      .create_application_command(|c| c.name("shock")
      .description("If you are shocked")
    )
      .create_application_command(|c| c.name("nervous")
      .description("Feeling nervous")
    )
      .create_application_command(|c| c.name("sad")
      .description("Feeling sad")
    )
      .create_application_command(|c| c.name("happy")
      .description("Feeling happy")
    )
      .create_application_command(|c| c.name("annoyed")
      .description("Really annoyed")
    )
      .create_application_command(|c| c.name("omg")
      .description("Oh my gawd")
    )
      .create_application_command(|c| c.name("smile")
      .description("Do a smile")
    )
      .create_application_command(|c| c.name("ew")
      .description("When you don't like something really")
    )
      .create_application_command(|c| c.name("awkward")
      .description("Feeling awkward")
    )
      .create_application_command(|c| c.name("oops")
      .description("This is just oops emotion...")
    )
      .create_application_command(|c| c.name("lazy")
      .description("Feeling lazy")
    )
      .create_application_command(|c| c.name("hungry")
      .description("Feeling hungry")
    )
      .create_application_command(|c| c.name("stressed")
      .description("Feeling stressed")
    )
      .create_application_command(|c| c.name("scared")
      .description("Really scared")
    )
      .create_application_command(|c| c.name("bored")
      .description("Feeling bored")
    )
      .create_application_command(|c| c.name("yes")
      .description("Yes Yes Yes")
    )
      .create_application_command(|c| c.name("no")
      .description("No No No")
    )
      .create_application_command(|c| c.name("bye")
      .description("Bye Bye")
    )
      .create_application_command(|c| c.name("sorry")
      .description("I am so sorry")
    )
      .create_application_command(|c| c.name("sleepy")
      .description("Feeling sleepy zzz")
    )
      .create_application_command(|c| c.name("wink")
      .description("Close and open one eye quickly")
    )
      .create_application_command(|c| c.name("facepalm")
      .description("A palm of a hand is brought to a face as an expression of dismay")
    )
      .create_application_command(|c| c.name("whatever")
      .description("you don't care")
    )
      .create_application_command(|c| c.name("pout")
      .description("do weird thing with lips")
    )
      .create_application_command(|c| c.name("smug")
      .description("showing an excessive pride in oneself")
    )
      .create_application_command(|c| c.name("smirk")
      .description("smile in an irritatingly smug, conceited, or silly way")
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
      .create_application_command(|c| c.name("pat")
      .description("Literally pat someone")
      .create_option(|o| {
          o.name("person")
          .description("Person to pat")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("slap")
      .description("Literally slap someone")
      .create_option(|o| {
          o.name("person")
          .description("Person to slap")
          .kind(ApplicationCommandOptionType::String)
          .required(true)
      })
    )
      .create_application_command(|c| c.name("gif")
      .description("Do some specific animation")
      .create_option(|o| {
          o.name("animation")
          .description("Search for specific animation")
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
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            }).await {
              error!("Failed to set DeferredChannelMessageWithSource {:?}", why);
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
              "clap" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Clapping ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "clap anime"
                                    , 0x22c2c1
                                    , images::own("clapping")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "angry" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Angry ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "angry anime"
                                    , 0x424203
                                    , images::own("Angry!")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "confused" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Confused ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "confused anime"
                                    , 0x3292e3
                                    , images::own("Confused...")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "nervous" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Nervous !!!")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "nervous anime"
                                    , 0x3252e3
                                    , images::own("Feeling nervous")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "sleepy" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("sleepy zzzzz")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "sleepy anime"
                                    , 0x3f22a3
                                    , images::own("Feeling sleepy")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "annoyed" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("annoyed ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "annoyed anime"
                                    , 0xaf52e3
                                    , images::own("Annoyed")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "sad" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("sad :(")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "sad anime"
                                    , 0x3252e3
                                    , images::own("Feeling sad :(")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "happy" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("happy =)")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "happy anime"
                                    , 0x6252e3
                                    , images::own("Feeling happy")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "shrug" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Shrugs ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "shrug anime"
                                    , 0x3252e3
                                    , images::own(r"shrugs ¯\_(ツ)_/¯")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "ahegao" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Ahegao")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "ahegao anime"
                                    , 0xf252c1
                                    , images::GType::Nothing
                                    , true, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "dance" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Dance Dance Dance")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "dancing anime"
                                    , 0x22b2c1
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "wink" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Winks")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "wink anime"
                                    , 0xaf52e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "facepalm" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Facepalm")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "facepalm anime"
                                    , 0x6f52f3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "pout" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "pout anime"
                                    , 0x6f52f3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "smug" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("=)")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "smug anime"
                                    , 0x6f52f3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "smirk" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content(";-)")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "smirk anime"
                                    , 0x6f52f3
                                    , images::own("smirks")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "whatever" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("whatever ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "whatever anime"
                                    , 0x6f52f3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "smile" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Doing the best smile")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "smile anime"
                                    , 0xafb2e3
                                    , images::own("smiles")
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "shock" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Shocked")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "dancing anime"
                                    , 0x722223
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "awkward" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Awkward")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "awkward anime"
                                    , 0xaf52e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "oops" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Oops")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "oops anime"
                                    , 0xaf52e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "lazy" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Feeling lazy")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "lazy anime"
                                    , 0xaf52e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "hungry" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Feeling hungry")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "hungry anime"
                                    , 0xaf5233
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "stressed" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Feeling stressed")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "stressed anime"
                                    , 0x2a62e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "scared" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Feeling scared")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "scared anime"
                                    , 0x2a3211
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "bored" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Feeling bored")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "bored anime"
                                    , 0xad52c3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "yes" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Yes")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "yes anime"
                                    , 0xbd22c3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "no" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("No")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "no anime"
                                    , 0x0d22f3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "bye" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Bye Bye")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "bye anime"
                                    , 0xad4213
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "sorry" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("I am sorry")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "sorry anime"
                                    , 0xcd5253
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "ew" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("ew...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "ew anime"
                                    , 0xaf82e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "omg" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("OMG")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "omg anime"
                                    , 0xaf52e3
                                    , images::GType::Nothing
                                    , false, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "lol" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("lol")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "lol anime"
                                    , 0xa656e3
                                    , images::GType::Nothing
                                    , true, None).await {
                        error!("Failed do gif emoji {:?}", err);
                      }
                    }, Err(why) => {
                      error!("Failed to create help interaction response {:?}", why);
                    }
                  };
                }
              },
              "cringe" => {
                if let Some(member) = &interaction.member {
                  match interaction.edit_original_interaction_response(&ctx.http, |response|
                    response.content("Cringe ...")
                  ).await {
                    Ok(mut msg) => {
                      if let Err(err) =
                        images::gifs( ctx, &member.user
                                    , &mut msg
                                    , "cringe"
                                    , 0x111111
                                    , images::GType::Nothing
                                    , true, None).await {
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
              "pat" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      if let Some(member) = &interaction.member {
                        match interaction.edit_original_interaction_response(&ctx.http, |response|
                          response.content("Pats ...")
                        ).await {
                          Ok(mut msg) => {
                            if let Err(err) =
                              images::gifs( ctx, &member.user
                                          , &mut msg
                                          , "pat anime"
                                          , 0x27e6d9
                                          , images::target("pats")
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
              "slap" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      if let Some(member) = &interaction.member {
                        match interaction.edit_original_interaction_response(&ctx.http, |response|
                          response.content("Slaps ...")
                        ).await {
                          Ok(mut msg) => {
                            if let Err(err) =
                              images::gifs( ctx, &member.user
                                          , &mut msg
                                          , "slap anime"
                                          , 0xd62929
                                          , images::target("slaps")
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
              "gif" => {
                if let Some(o) = ac.options.first() {
                  if let Some(v) = o.value.clone() {
                    if let Some(t) = v.as_str() {

                      if let Some(member) = &interaction.member {
                        match interaction.edit_original_interaction_response(&ctx.http, |response|
                          response.content("Searching ...")
                        ).await {
                          Ok(mut msg) => {
                            if let Err(err) =
                              images::gifs( ctx, &member.user
                                          , &mut msg
                                          , t, 0x8e613b
                                          , images::GType::Nothing
                                          , true, None ).await {
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
                  Ok(mut msg) => {
                    let args = Args::new(str_arg.as_str(), &[Delimiter::Single(';')]);
                    if let Err(terr) = meta::time_slash(&ctx, &mut msg, args).await {
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

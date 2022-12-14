use crate::{
  message::RESTORE,
  common::i18n::{ edit_help_i18n, US_ENG },
  commands::{
    w3c::stats,
    chat, meta, music, images
  }
};

#[cfg(not(target_os = "windows"))]
use crate::{
  steins::{ ai::boris
          , ai::uwu },
  commands::translation
};

use serenity::{
  prelude::*,
  builder::*,
  model::application::interaction::{ // InteractionResponseType
                                     Interaction },
  framework::standard::{ Args, Delimiter }
};

use serenity::model::application::interaction::application_command::{
  ResolvedOption,
  ResolvedValue
};

use std::sync::atomic::Ordering;

static ASYNC_CMDS: [&str; 51] = [ "translate", "перевод", "help"
                                , "stats", "феминизировать"
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

pub async fn handle_slash_commands(ctx: &Context, interaction: &Interaction) {
  if let Interaction::Command(ac) = interaction {
    match ac.data.name.as_str() {
      "join" => {
        if let Some(guild_id) = &ac.guild_id {
          let guild = if let Some(guild) = guild_id.to_guild_cached(ctx) {
            guild.clone()
          } else {
            return;
          };
          if let Some(member) = &ac.member {
            if let Err(err) = music::join_slash(ctx, &member.user, &guild).await {
              if let Err(why) = ac.create_response(&ctx.http, CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default().content( format!("Failed to join {err}") )
                  )
              ).await {
                error!("Failed to create boris interaction response {why}");
              }
            } else if let Err(why) =
              ac.create_response(&ctx.http, CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default().content( "I've joined voice channel" )
                )
            ).await {
              error!("Failed to create boris interaction response {why}");
            }
          }
        }
      }
      "борис" => {
        if let Some(ResolvedOption {
          value: ResolvedValue::String(t), ..
        }) = ac.data.options().first() {
          #[cfg(not(target_os = "windows"))]
          if let Err(why) = ac.create_response(&ctx.http, CreateInteractionResponse::Message(
              CreateInteractionResponseMessage::default().content( boris::spell(t) ) )
          ).await {
            error!("Failed to create boris interaction response {why}");
          }
        }
      },
      "uwu" => {
        if let Some(ResolvedOption {
          value: ResolvedValue::String(t), ..
        }) = ac.data.options().first() {
          #[cfg(not(target_os = "windows"))]
          if let Err(why) = ac.create_response(&ctx.http, CreateInteractionResponse::Message(
              CreateInteractionResponseMessage::default().content( uwu::spell(t) ) )
          ).await {
            error!("Failed to create uwu interaction response {why}");
          }
        }
      },
      c if ASYNC_CMDS.iter().any(|cmd| c == *cmd) => {

        if let Err(why) = ac.create_response(&ctx.http,
          CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default())
        ).await {
          error!("Failed to set DeferredChannelMessageWithSource {why}");
        }

        match c {
          "help" => {
            match ac.edit_response(&ctx.http, EditInteractionResponse::default()
              .content("Creating Help ...")
            ).await {
              Ok(mut msg) => {
                edit_help_i18n(ctx, &mut msg, &US_ENG).await;
              }, Err(why) => {
                error!("Failed to create help interaction response {why}");
              }
            };
          },
          "wave" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Waving ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "wave anime"
                                , 0x3252e3i32
                                , images::own("waves")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "cry" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Crying ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "cry anime"
                                , 0x126223i32
                                , images::own("crying")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "clap" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Clapping ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "clap anime"
                                , 0x22c2c1i32
                                , images::own("clapping")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "angry" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Angry ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "angry anime"
                                , 0x424203i32
                                , images::own("Angry!")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "confused" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Confused ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "confused anime"
                                , 0x3292e3i32
                                , images::own("Confused...")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "nervous" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Nervous !!!")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "nervous anime"
                                , 0x3252e3i32
                                , images::own("Feeling nervous")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "sleepy" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("sleepy zzzzz")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "sleepy anime"
                                , 0x3f22a3i32
                                , images::own("Feeling sleepy")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "annoyed" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("annoyed ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "annoyed anime"
                                , 0xaf52e3i32
                                , images::own("Annoyed")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "sad" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("sad :(")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "sad anime"
                                , 0x3252e3i32
                                , images::own("Feeling sad :(")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "happy" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("happy =)")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "happy anime"
                                , 0x6252e3i32
                                , images::own("Feeling happy")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "shrug" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Shrugs ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "shrug anime"
                                , 0x3252e3i32
                                , images::own(r"shrugs ¯\_(ツ)_/¯")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "ahegao" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Ahegao")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "ahegao anime"
                                , 0xf252c1i32
                                , images::GType::Nothing
                                , true, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "dance" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Dance Dance Dance")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "dancing anime"
                                , 0x22b2c1i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "wink" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Winks")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "wink anime"
                                , 0xaf52e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "facepalm" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Facepalm")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "facepalm anime"
                                , 0x6f52f3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "pout" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "pout anime"
                                , 0x6f52f3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "smug" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("=)")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "smug anime"
                                , 0x6f52f3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "smirk" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content(";-)")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "smirk anime"
                                , 0x6f52f3i32
                                , images::own("smirks")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "whatever" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("whatever ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "whatever anime"
                                , 0x6f52f3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "smile" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Doing the best smile")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "smile anime"
                                , 0xafb2e3i32
                                , images::own("smiles")
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "shock" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Shocked")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "dancing anime"
                                , 0x722223i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "awkward" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Awkward")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "awkward anime"
                                , 0xaf52e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "oops" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Oops")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "oops anime"
                                , 0xaf52e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "lazy" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Feeling lazy")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "lazy anime"
                                , 0xaf52e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "hungry" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Feeling hungry")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "hungry anime"
                                , 0xaf5233i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "stressed" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Feeling stressed")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "stressed anime"
                                , 0x2a62e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "scared" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Feeling scared")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "scared anime"
                                , 0x2a3211i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "bored" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Feeling bored")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "bored anime"
                                , 0xad52c3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "yes" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Yes")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "yes anime"
                                , 0xbd22c3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "no" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("No")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "no anime"
                                , 0x0d22f3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "bye" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Bye Bye")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "bye anime"
                                , 0xad4213i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "sorry" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("I am sorry")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "sorry anime"
                                , 0xcd5253i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "ew" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("ew...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "ew anime"
                                , 0xaf82e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "omg" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("OMG")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "omg anime"
                                , 0xaf52e3i32
                                , images::GType::Nothing
                                , false, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "lol" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("lol")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "lol anime"
                                , 0xa656e3i32
                                , images::GType::Nothing
                                , true, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "cringe" => {
            if let Some(member) = &ac.member {
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content("Cringe ...")
              ).await {
                Ok(mut msg) => {
                  if let Err(err) =
                    images::gifs( ctx, &member.user
                                , &mut msg
                                , "cringe"
                                , 0x111111i32
                                , images::GType::Nothing
                                , true, None).await {
                    error!("Failed do gif emoji {err}");
                  }
                }, Err(why) => {
                  error!("Failed to create help interaction response {why}");
                }
              };
            }
          },
          "hug" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              if let Some(member) = &ac.member {
                match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                  .content("Hugs ...")
                ).await {
                  Ok(mut msg) => {
                    if let Err(err) =
                      images::gifs( ctx, &member.user
                                  , &mut msg
                                  , "hug anime"
                                  , 0xed9e2fi32
                                  , images::target("hugs")
                                  , false, Some( String::from(*t) )).await {
                      error!("Failed do gif emoji {err}");
                    }
                  }, Err(why) => {
                    error!("Failed to create help interaction response {why}");
                  }
                };
              }
            }
          },
          "pat" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              if let Some(member) = &ac.member {
                match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                  .content("Pats ...")
                ).await {
                  Ok(mut msg) => {
                    if let Err(err) =
                      images::gifs( ctx, &member.user
                                  , &mut msg
                                  , "pat anime"
                                  , 0x27e6d9i32
                                  , images::target("pats")
                                  , false, Some( String::from(*t) )).await {
                      error!("Failed do gif emoji {err}");
                    }
                  }, Err(why) => {
                    error!("Failed to create help interaction response {why}");
                  }
                };
              }
            }
          },
          "slap" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              if let Some(member) = &ac.member {
                match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                  .content("Slaps ...")
                ).await {
                  Ok(mut msg) => {
                    if let Err(err) =
                      images::gifs( ctx, &member.user
                                  , &mut msg
                                  , "slap anime"
                                  , 0xd62929i32
                                  , images::target("slaps")
                                  , false, Some( String::from(*t) )).await {
                      error!("Failed do gif emoji {err}");
                    }
                  }, Err(why) => {
                    error!("Failed to create help interaction response {why}");
                  }
                };
              }
            }
          },
          "gif" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              if let Some(member) = &ac.member {
                match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                  .content("Searching ...")
                ).await {
                  Ok(mut msg) => {
                    if let Err(err) =
                      images::gifs( ctx, &member.user
                                  , &mut msg
                                  , t, 0x8e613bi32
                                  , images::GType::Nothing
                                  , true, None ).await {
                      error!("Failed do gif emoji {err}");
                    }
                  }, Err(why) => {
                    error!("Failed to create help interaction response {why}");
                  }
                };
              }
            }
          },
          "leave" => {
            match ac.edit_response(&ctx.http, EditInteractionResponse::default()
              .content("Leaving...")
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
            match ac.edit_response(&ctx.http, EditInteractionResponse::default()
              .content("Repeating...")
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
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              RESTORE.store(false, Ordering::Relaxed);
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content(&format!("Getting stats for {t}"))
              ).await {
                Ok(msg) => {
                  let args = Args::new(t, &[Delimiter::Single(';')]);
                  if let Err(serr) = stats(ctx, &msg, args).await {
                    error!("Failed to get stats on interaction {serr:?}");
                  }
                }, Err(why) => {
                  error!("Failed to create stats interaction response {why:?}");
                }
              };
              RESTORE.store(true, Ordering::Relaxed);
            }
          },
          "феминизировать" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              RESTORE.store(false, Ordering::Relaxed);
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content(&format!("Феминизация \"{t}\""))
              ).await {
                Ok(msg) => {
                  let args = Args::new(t, &[Delimiter::Single(';')]);
                  if let Err(serr) = chat::feminize(ctx, &msg, args).await {
                    error!("Failed to feminize on interaction {serr:?}");
                  }
                }, Err(why) => {
                  error!("Failed to feminize on interaction response {why:?}");
                }
              };
              RESTORE.store(true, Ordering::Relaxed);
            }
          },
          "play" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              RESTORE.store(false, Ordering::Relaxed);
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content(&format!("Playing {t}"))
              ).await {
                Ok(msg) => {
                  let args = Args::new(t, &[Delimiter::Single(';')]);
                  if let Err(serr) = music::play(ctx, &msg, args).await {
                    error!("Failed to get play on interaction {serr:?}");
                  }
                }, Err(why) => {
                  error!("Failed to create play interaction response {why:?}");
                }
              };
              RESTORE.store(true, Ordering::Relaxed);
            }
          },
          cmd if cmd == "translate" || cmd == "перевод" => {
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              RESTORE.store(false, Ordering::Relaxed);
              match ac.edit_response(&ctx.http, EditInteractionResponse::default()
                .content(&format!("Translating {t}"))
              ).await {
                Ok(msg) => {
                  let args = Args::new(t, &[Delimiter::Single(';')]);
                  #[cfg(not(target_os = "windows"))]
                  if cmd == "translate" {
                    if let Err(terr) = translation::translate(ctx, &msg, args).await {
                      error!("Failed to translate to English on interaction {terr:?}");
                    }
                  } else if let Err(terr) = translation::perevod(ctx, &msg, args).await {
                    error!("Failed to translate to Russian on interaction {terr:?}");
                  }
                }, Err(why) => {
                  error!("Failed to create translation interaction response {why:?}");
                }
              };
              RESTORE.store(true, Ordering::Relaxed);
            }
          },
          cmd if cmd == "time" || cmd == "время" => {
            let mut str_arg = String::new();
            if let Some(ResolvedOption {
              value: ResolvedValue::String(t), ..
            }) = ac.data.options().first() {
              str_arg = String::from(*t);
            }
            RESTORE.store(false, Ordering::Relaxed);
            match ac.edit_response(&ctx.http, EditInteractionResponse::default()
              .content("⌚")
            ).await {
              Ok(mut msg) => {
                let args = Args::new(str_arg.as_str(), &[Delimiter::Single(';')]);
                if let Err(terr) = meta::time_slash(ctx, &mut msg, args).await {
                  error!("Failed to show time on interaction {terr:?}");
                }
              }, Err(why) => {
                error!("Failed to show time interaction response {why:?}");
              }
            };
            RESTORE.store(true, Ordering::Relaxed);
          },
          _ => { /* dunno */ }
        };
      },
      _ => { /* dunno */ }
    };
  }
}

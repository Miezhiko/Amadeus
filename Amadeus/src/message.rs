use crate::{
  types::{ serenity::CoreGuild
         , options::* },
  steins::{ gate
          , warcraft::{
              replay::{ replay_embed
                      , attach_replay }
            }
          },
  common::{ db::trees::points
          , help::lang
          , msg::channel_message
          , constants::{ PREFIX, UNBLOCK_ROLE }
          },
  collections::{ base::{ REACTIONS, WHITELIST }
               , channels::{ AI_ALLOWED, IGNORED }
               , team::DISCORDS
               }
};

#[cfg(feature = "spam_filter")]
use crate::spam::spam_check;

#[cfg(not(target_os = "windows"))]
use crate::steins::ai::{ cache, chain, response };

use serenity::{
  prelude::*,
  builder::*,
  model::{ id::{ EmojiId, MessageId, UserId }
         , channel::{ Message, ReactionType }
         , colour::Colour
         },
  gateway::ActivityData
};

use std::{ collections::VecDeque
         , sync::atomic::{ Ordering, AtomicBool }
         , time::Duration
         , borrow::Borrow
         };

use rand::{ Rng
          , seq::SliceRandom
          , rngs::StdRng
          , SeedableRng };

use regex::Regex;
use once_cell::sync::Lazy;

pub static RESTORE: AtomicBool  = AtomicBool::new(false);

pub static BACKUP: Lazy<Mutex<VecDeque<(MessageId, Message)>>> =
  Lazy::new(|| Mutex::new(VecDeque::with_capacity(64)));

pub async fn process( ioptions: &IOptions
                    , amadeus_id: UserId
                    , ctx: &Context
                    , msg: Message ) {

  if msg.is_own(ctx) {
    if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0.get()) {
      let mut backup_deq = BACKUP.lock().await;
      if backup_deq.len() == backup_deq.capacity() {
        backup_deq.pop_front();
      }
      backup_deq.push_back((msg.id, msg));
    }
  } else if msg.author.bot {
    if let Some(g) = msg.guild_id {
      if ioptions.servers.iter()
                         .any(|s| s.id    == g.0.get()
                               && s.kind  == CoreGuild::Safe) {
        return;
      }
    }
    if IGNORED.contains(&msg.channel_id.0.get()) {
      return;
    }
    let mut is_file = false;
    for file in &msg.attachments {
      if let Ok(bytes) = file.download().await {
        let cow = CreateAttachment::bytes(
          bytes,
          String::from(&file.filename)
        );
        if let Err(why) = msg.channel_id.send_message(&ctx, CreateMessage::new().add_file(cow)).await {
          error!("Failed to download and post attachment {:?}", why);
        } else {
          is_file = true;
        }
      }
    }
    if let Err(why) = &msg.delete(&ctx).await {
      error!("Error replacing other bots {:?}", why);
    }
    if is_file {
      if let Ok(messages) = msg.channel_id.messages(&ctx, GetMessages::default()
        .limit(3)
      ).await {
        for mmm in messages {
          if mmm.content.to_lowercase().contains("processing") {
            if let Err(why) = mmm.delete(&ctx).await {
              error!("Error removing processing message {:?}", why);
            }
          }
        }
      }
    }
    if !msg.content.is_empty() && !msg.content.starts_with("http") {
      channel_message(ctx, &msg, &msg.content).await;
    }
    for embed in &msg.embeds {
      let mut not_stupid_zephyr = true;

      if let Some(desc) = &embed.description {
        if desc.contains("DiscordAPIError")
        || desc.contains("Unknown Message") {
          not_stupid_zephyr = false;
        }
      }
      if not_stupid_zephyr {
        if let Err(why) = &msg.channel_id.send_message(&ctx, CreateMessage::new()
          .embed( CreateEmbed::from( embed.clone() ) )
        ).await {
          error!("Error replacing other bots embeds {why}");
        }
      }
    }
  } else if !msg.content.starts_with(PREFIX) {
    if let Some(guild_id) = msg.guild_id {
      #[cfg(feature = "spam_filter")]
      spam_check(&guild_id, ctx, &msg).await;
      if msg.mentions.iter().any(|u| u.bot) {
        if msg.mentions.iter().any(|u| u.bot && u.id == amadeus_id) {
          set!{ amention1 = format!("<@{}>", amadeus_id)
              , amention2 = format!("<@!{}>", amadeus_id) };
          if !msg.content.starts_with(&amention1)
          && !msg.content.starts_with(&amention2) {
            #[cfg(not(target_os = "windows"))]
            response::response(ctx, &msg).await;
          }
        }
      } else {
        points::add_points(guild_id.0.get(), msg.author.id.0.get(), 1).await;
        let msg_channel_id_u64 = msg.channel_id.0.get();
        for file in &msg.attachments {
          if file.filename.ends_with("w3g") {
            if DISCORDS.iter().any(|(_,df)| df.games.unwrap_or(0)  == msg_channel_id_u64
                                         || df.games2.unwrap_or(0) == msg_channel_id_u64
                                         || df.games4.unwrap_or(0) == msg_channel_id_u64) {
              if attach_replay(ctx, &msg, file).await.is_ok() {
                info!("Relay attached successfully");
              } else {
                warn!("Failed to attach an replay to log!");
              }
              if let Err(why) = &msg.delete(&ctx).await {
                error!("failed to clean attachment from log {why}");
              }
            } else {
              let rainbow = ReactionType::Unicode(String::from("🌈"));
              let _ = msg.react(&ctx, rainbow).await;
              loop {
                let collector = msg.reaction_collector(&ctx.shard)
                                   .timeout(Duration::from_secs(3600));
                if let Some(reaction) = collector.collect_single().await {
                  let emoji = &reaction.emoji;
                  // TODO: rework this, use match on &ReactionType somehow
                  match emoji.as_data().as_str() {
                    "%F0%9F%8C%88" => {
                      info!("parsing replay");
                      if let Err(why) = replay_embed(ctx, &msg, file).await {
                        error!("Failed to analyze replay:\n{why}");
                      }
                      if let Err(why) = msg.delete_reactions(ctx).await {
                        error!("failed to delte msg reactions {why}");
                      }
                    },
                    _ => ()
                  }
                }
              }
            }
            return;
          }
        }
        // any other junk on log channel should be removed
        if DISCORDS.iter().any(|(_,df)| df.games.unwrap_or(0)  == msg_channel_id_u64
                                     || df.games2.unwrap_or(0) == msg_channel_id_u64
                                     || df.games4.unwrap_or(0) == msg_channel_id_u64) {
          if let Err(why) = &msg.delete(&ctx).await {
            error!("failed to clean junk from log {why}");
          }
          return;
        }
        gate::LAST_CHANNEL.store(msg.channel_id.0.get(), Ordering::Relaxed);

        let rndx: u8 = rand::thread_rng().gen_range(0..3);
        if rndx != 1 {
          if let Some(nick) = msg.author.nick_in(ctx, &guild_id).await {
            ctx.set_activity(Some( ActivityData::listening(&nick) ));
          } else {
            ctx.set_activity(Some( ActivityData::listening(&msg.author.name) ));
          }
          ctx.online();
        } else {
          #[cfg(target_os = "windows")]
          let activity = "doing nothing";
          #[cfg(not(target_os = "windows"))]
          let activity = chain::generate(ctx, &msg, None).await;
          if !activity.is_empty() {
            if activity.contains('<') && activity.contains('>') {
              static RE_IB: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
              let replaced = RE_IB.replace_all(&activity, "");
              if !replaced.is_empty() {
                ctx.set_activity(Some( ActivityData::competing(&*replaced) ));
              }
            } else {
              ctx.set_activity(Some( ActivityData::playing(activity) ));
            }
            ctx.idle();
          }
        }
        #[cfg(not(target_os = "windows"))]
        if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0.get()) {
          let activity_level = cache::ACTIVITY_LEVEL.load(Ordering::Relaxed);
          let rnd = rand::thread_rng().gen_range(0..activity_level);
          if rnd == 1 {
            response::chat(ctx, &msg).await;
          }
          let rnd2: u16 = rand::thread_rng().gen_range(0..2);
          if rnd2 == 1 {
            let mut rng = StdRng::from_entropy();
            let emoji = REACTIONS.choose(&mut rng).unwrap();
            let reaction = ReactionType::Custom {
              animated: false,
              id: EmojiId( to_nzu!(emoji.id) ),
              name: Some(emoji.name.clone())
            };

            if let Ok(guild) = guild_id.to_partial_guild(ctx).await {
              if let Ok(mut member) = guild.member(ctx, msg.author.id).await {
                if let Some(role) = guild.role_by_name(UNBLOCK_ROLE) {

                  let normal_people_rnd: u16 = rand::thread_rng().gen_range(0..9);
                  if (normal_people_rnd == 1 || member.roles.contains(&role.id))
                  && !WHITELIST.iter().any(|u| *u == msg.author.id.0.get())
                  && !ioptions.servers.iter()
                                      .any(|s| s.id    == guild_id.0.get()
                                            && s.kind  == CoreGuild::Safe) {
                    if let Err(why) = msg.react(ctx, reaction).await {
                      error!("Failed to react with {}: {why}", emoji.name);
                      if why.to_string().contains("blocked")
                      && !member.roles.contains(&role.id) {
                        if let Err(why) = member.add_role(&ctx, role).await {
                          error!("Failed to assign hater role {why}");
                        } else {
                          let nick = member.nick.unwrap_or_else(|| msg.author.name.clone());
                          let repl = if lang::is_russian(&msg.content) {
                            format!("Ну чел {} явно меня не уважает", &nick)
                          } else {
                            format!("Seems like {} doesn't respect me :(", &nick)
                          };
                          channel_message(ctx, &msg, &repl).await;
                          let new_nick: String = format!("Hater {}", &msg.author.name);
                          if let Err(why2) = guild_id.edit_member(ctx, msg.author.id, EditMember::default()
                            .nickname(new_nick)).await {
                            error!("Failed to change user's nick {why2}");
                          }
                        }
                      }
                    } else if member.roles.contains(&role.id) {
                      if let Err(why) = member.remove_role(ctx, role).await {
                        error!("Failed to remove gay role {why}");
                      } else {
                        let repl = if lang::is_russian(&msg.content) {
                          format!("Пчел {} извини если что, давай останемся друзьями", msg.author.name)
                        } else {
                          format!("Dear {} thank you for unblocking me, let be friends!", msg.author.name)
                        };
                        channel_message(ctx, &msg, &repl).await;
                        if let Err(why2) = guild_id.edit_member(ctx, msg.author.id, EditMember::default().nickname("")).await {
                          error!("Failed to reset user's nick {why2}");
                        }
                      }
                    }
                  }

                  if member.roles.contains(&role.id) {
                    let new_nick = format!("Hater {}", msg.author.name);
                    if let Err(why2) = guild_id.edit_member(ctx, msg.author.id, EditMember::default().nickname(new_nick)).await {
                      error!("Failed to change user's nick {why2}");
                    }
                    if let Err(why) = &msg.delete(ctx).await {
                      error!("Error replacing bad people {why}");
                    }
                    if !msg.content.is_empty() && !msg.content.starts_with("http") {
                      let new_words = chain::obfuscate(&msg.content);
                      let says = if lang::is_russian(&new_words) {
                        "говорит"
                      } else { "says" };
                      let rm = format!("{} {} {} {}", msg.author.name
                                                    , says, new_words
                                                    , &msg.content);
                      channel_message(ctx, &msg, &rm).await;
                    }
                  }

                  let rnd3: u8 = rand::thread_rng().gen_range(0..9);
                  if rnd3 != 1 {
                    if let Err(why) = msg.delete_reactions(ctx).await {
                      error!("Failed to remove all the reactions {why}");
                    }
                  }
                } else if let Err(why) =
                  guild.create_role(&ctx, EditRole::default()
                       .colour(Colour::from_rgb(226,37,37).0)
                       .name(UNBLOCK_ROLE)).await {
                  error!("Failed to create UNBLOCK role, {why}");
                }
              }
            }
          }
        }
      }
    }
  }
}

use crate::{
  types::{ common::CoreGuild
         , options::* },
  steins::{ gate
          , ai::{ cache, chain }
          , cyber::replay::{ replay_embed
                           , attach_replay }
          },
  common::{ db::trees
          , help::lang
          , msg::channel_message
          , constants::{ LOG_CHANNEL
                       , UNBLOCK_ROLE }
          },
  collections::{ base::{ REACTIONS, WHITELIST }
               , channels::{ AI_ALLOWED, EXCEPTIONS, IGNORED }
               }
};

use serenity::{
  prelude::*,
  utils::Colour,
  model::{ id::{ EmojiId, MessageId, UserId }
         , channel::Message, channel::ReactionType, gateway::Activity
         },
  http::AttachmentType,
  builder::CreateEmbed
};

use std::{ borrow::Cow, collections::VecDeque
         , sync::atomic::{ Ordering, AtomicBool }
         , time::Duration
         };

use rand::{ Rng
          , seq::SliceRandom
          , rngs::StdRng
          , SeedableRng };

use regex::Regex;
use once_cell::sync::Lazy;

pub static BLAME: AtomicBool    = AtomicBool::new(false);
pub static RESTORE: AtomicBool  = AtomicBool::new(false);

pub static BACKUP: Lazy<Mutex<VecDeque<(MessageId, Message)>>> =
  Lazy::new(|| Mutex::new(VecDeque::with_capacity(64)));

pub async fn process( ioptions: &IOptions
                    , amadeus_id: UserId
                    , ctx: &Context
                    , msg: Message) {

  if msg.is_own(&ctx).await {
    let blame_check = BLAME.load(Ordering::Relaxed);
    if !blame_check {
      if let Some(g) = msg.guild_id {
        if ioptions.servers.iter().any(|s| s.id == g.0
                                        && s.kind == CoreGuild::Unsafe) {
          if let Ok(guild) = g.to_partial_guild(&ctx).await {
            if let Ok(member) = guild.member(&ctx, &msg.author.id).await {
              if let Ok(some_permissions) = member.permissions(&ctx).await {
                if !some_permissions.administrator() {
                  BLAME.store(true, Ordering::Relaxed);
                  for _ in 0..10u8 {
                    channel_message(&ctx, &msg,
                      "Set administrator role for me, please!").await;
                  }
                  BLAME.store(false, Ordering::Relaxed);
                }
              }
            }
          }
        }
      }
      if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0) {
        let mut backup_deq = BACKUP.lock().await;
        if backup_deq.len() == backup_deq.capacity() {
          backup_deq.pop_front();
        }
        backup_deq.push_back((msg.id, msg));
      }
    }
  } else if msg.author.bot {
    if let Some(g) = msg.guild_id {
      if ioptions.servers.iter()
                         .any(|s| s.id == g.0
                               && s.kind == CoreGuild::Safe) {
        return;
      }
    }
    if EXCEPTIONS.contains(&msg.channel_id.0) {
      return;
    }
    let mut is_file = false;
    for file in &msg.attachments {
      if let Ok(bytes) = file.download().await {
        let cow = AttachmentType::Bytes {
          data: Cow::from(bytes),
          filename: String::from(&file.filename)
        };
        if let Err(why) = msg.channel_id.send_message(&ctx, |m| m.add_file(cow)).await {
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
      if let Ok(messages) = msg.channel_id.messages(&ctx, |r|
        r.limit(3)
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
      channel_message(&ctx, &msg, &msg.content).await;
    }
    for embed in &msg.embeds {
      let mut not_stupid_zephyr = true;
      
      if let Some(desc) = &embed.description {
        if desc.contains("DiscordAPIError") {
          not_stupid_zephyr = false;
        }
      }
      if not_stupid_zephyr {
        if let Err(why) = &msg.channel_id.send_message(&ctx, |m| {
          m.embed(|e| {
            *e = CreateEmbed::from(embed.clone());
            e })
        }).await {
          error!("Error replacing other bots embeds {:?}", why);
        }
      }
    }
  } else if !msg.content.starts_with('~') {
    if let Some(guild_id) = msg.guild_id {
      if IGNORED.contains(&msg.channel_id.0) {
        return;
      }
      if (&msg.mentions).iter().any(|u| u.bot) {
        if (&msg.mentions).iter().any(|u| u.bot && u.id == amadeus_id) {
          let amention1 = format!("<@{}>", amadeus_id);
          let amention2 = format!("<@!{}>", amadeus_id);
          if !msg.content.starts_with(&amention1)
          && !msg.content.starts_with(&amention2) {
            chain::response(&ctx, &msg).await;
          }
        }
      } else {
        trees::add_points(guild_id.0, msg.author.id.0, 1).await;
        for file in &msg.attachments {
          if file.filename.ends_with("w3g") {
            if msg.channel_id == LOG_CHANNEL {
              if !attach_replay(&ctx, &msg, file).await {
                warn!("Failed to attach an replay to log!");
              } else {
                info!("Relay attached successfully");
              }
              if let Err(why) = &msg.delete(&ctx).await {
                error!("failed to clean attachment from log {:?}", why);
              }
            } else {
              let rainbow = ReactionType::Unicode(String::from("🌈"));
              let _ = msg.react(&ctx, rainbow).await;
              loop {
                if let Some(reaction) =
                  &msg.await_reaction(&ctx)
                      .timeout(Duration::from_secs(3600)).await {
                  let emoji = &reaction.as_inner_ref().emoji;
                  if emoji.as_data().as_str() == "🌈" {
                    if let Err(why) = replay_embed(&ctx, &msg, file).await {
                      error!("Failed to analyze replay:\n{:?}", why);
                    }
                    if let Err(why) = msg.delete_reactions(&ctx).await {
                      error!("failed to delte msg reactions {:?}", why);
                    }
                  }
                } else {
                  if let Err(why) = msg.delete_reactions(&ctx).await {
                    error!("failed to delte msg reactions {:?}", why);
                  }
                  break;
                }
              }
            }
            return;
          }
        }
        // any other junk on log channel should be removed
        if msg.channel_id == LOG_CHANNEL {
          if let Err(why) = &msg.delete(&ctx).await {
            error!("failed to clean junk from log {:?}", why);
          }
          return;
        }
        gate::LAST_CHANNEL.store(msg.channel_id.0, Ordering::Relaxed);

        let rndx: u8 = rand::thread_rng().gen_range(0..3);
        if rndx != 1 {
          if let Some(nick) = msg.author.nick_in(&ctx, &guild_id).await {
            ctx.set_activity(Activity::listening(&nick)).await;
          } else {
            ctx.set_activity(Activity::listening(&msg.author.name)).await;
          }
          ctx.online().await;
        } else {
          let activity = chain::generate(&ctx, &msg, None).await;
          if !activity.is_empty() {
            if activity.contains('<') && activity.contains('>') {
              static RE_IB: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"<(.*?)>").unwrap());
              let replaced = RE_IB.replace_all(&activity, "");
              if !replaced.is_empty() {
                ctx.set_activity(Activity::competing(&replaced)).await;
              }
            } else {
              ctx.set_activity(Activity::playing(&activity)).await;
            }
            ctx.idle().await;
          }
        }

        if AI_ALLOWED.iter().any(|c| c.id == msg.channel_id.0) {
          let activity_level = cache::ACTIVITY_LEVEL.load(Ordering::Relaxed);
          let rnd = rand::thread_rng().gen_range(0..activity_level);
          if rnd == 1 {
            chain::chat(&ctx, &msg).await;
          }
          let rnd2: u16 = rand::thread_rng().gen_range(0..2);
          if rnd2 == 1 {
            let mut rng = StdRng::from_entropy();
            let emoji = REACTIONS.choose(&mut rng).unwrap();
            let reaction = ReactionType::Custom {
              animated: false,
              id: EmojiId(emoji.id),
              name: Some(emoji.name.clone())
            };

            if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
              if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
                if let Some(role) = guild.role_by_name(UNBLOCK_ROLE) {

                  let normal_people_rnd: u16 = rand::thread_rng().gen_range(0..9);
                  if (normal_people_rnd == 1 || member.roles.contains(&role.id))
                  && !WHITELIST.iter().any(|u| *u == msg.author.id.0)
                  && !ioptions.servers.iter()
                                      .any(|s| s.id    == guild_id.0
                                            && s.kind  == CoreGuild::Safe) {

                    if let Err(why) = msg.react(&ctx, reaction).await {
                      error!("Failed to react: {:?}", why);
                      if why.to_string().contains("blocked")
                      && !member.roles.contains(&role.id) {
                        if let Err(why) = member.add_role(&ctx, role).await {
                          error!("Failed to assign hater role {:?}", why);
                        } else {
                          let nick = member.nick.unwrap_or_else(|| msg.author.name.clone());
                          let repl = if lang::is_russian(&msg.content) {
                            format!("Ну чел {} явно меня не уважает", &nick)
                          } else {
                            format!("Seems like {} doesn't respect me :(", &nick)
                          };
                          channel_message(&ctx, &msg, &repl).await;
                          let new_nick: String = format!("Hater {}", &msg.author.name);
                          if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m|
                            m.nickname(new_nick)).await {
                            error!("Failed to change user's nick {:?}", why2);
                          }
                        }
                      }
                    } else if member.roles.contains(&role.id) {
                      if let Err(why) = member.remove_role(&ctx, role).await {
                        error!("Failed to remove gay role {:?}", why);
                      } else {
                        let repl = if lang::is_russian(&msg.content) {
                          format!("Пчел {} извини если что, давай останемся друзьями", msg.author.name)
                        } else {
                          format!("Dear {} thank you for unblocking me, let be friends!", msg.author.name)
                        };
                        channel_message(&ctx, &msg, &repl).await;
                        if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m| m.nickname("")).await {
                          error!("Failed to reset user's nick {:?}", why2);
                        }
                      }
                    }
                  }

                  if member.roles.contains(&role.id) {
                    let new_nick = format!("Hater {}", msg.author.name);
                    if let Err(why2) = guild_id.edit_member(&ctx, msg.author.id, |m| m.nickname(new_nick)).await {
                      error!("Failed to change user's nick {:?}", why2);
                    }
                    if let Err(why) = &msg.delete(&ctx).await {
                      error!("Error replacing bad people {:?}", why);
                    }
                    if !msg.content.is_empty() && !msg.content.starts_with("http") {
                      let new_words = chain::obfuscate(&msg.content);
                      let says = if lang::is_russian(&new_words) {
                        "говорит"
                      } else { "says" };
                      let rm = format!("{} {} {} {}", msg.author.name, says, new_words, &msg.content);
                      channel_message(&ctx, &msg, &rm).await;
                    }
                  }

                  let rnd3: u8 = rand::thread_rng().gen_range(0..9);
                  if rnd3 != 1 {
                    if let Err(why) = msg.delete_reactions(&ctx).await {
                      error!("Failed to remove all the reactions {:?}", why);
                    }
                  }
                } else if let Err(why) =
                  guild.create_role(&ctx,
                      |r| r.colour(Colour::from_rgb(226,37,37).0 as u64)
                            .name(UNBLOCK_ROLE)).await {
                  error!("Failed to create UNBLOCK role, {:?}", why);
                }
              }
            }
          }
        }
      }
    }
  }
}

use crate::{
  types::options::*,
  steins::{ gate
          , ai::chain
          , cyber::w3g::analyze
          },
  common::{ points
          , help::{ lang, channel::channel_by_name }
          , msg::channel_message
          },
  collections::{ base::{ REACTIONS, WHITELIST, WHITELIST_SERVERS }
               , stuff::overwatch::{ OVERWATCH, OVERWATCH_REPLIES }
               , channels::{ AI_ALLOWED, IGNORED }
               },
  commands::voice
};

use serenity::{
  prelude::*,
  async_trait,
  model::{ guild::ActionMessage
         , id::{ EmojiId, GuildId, MessageId, UserId, ChannelId }
         , event::ResumedEvent, gateway::Ready, guild::Member
         , channel::Message, channel::ReactionType, gateway::Activity
         , user::User
         },
  http::AttachmentType,
  builder::CreateEmbed
};

use std::{ borrow::Cow, collections::VecDeque
         , sync::atomic::{ Ordering, AtomicBool }
         , time::Duration
         };

use async_std::{ fs::File, fs
               , prelude::* };

use rand::{ Rng
          , seq::SliceRandom
          , rngs::StdRng
          , SeedableRng };

use regex::Regex;

use plotters::prelude::*;

pub static THREADS: AtomicBool = AtomicBool::new(false);
pub static BLAME: AtomicBool = AtomicBool::new(false);
pub static RESTORE: AtomicBool = AtomicBool::new(false);

pub struct Handler {
  ioptions: IOptions,
  roptions: ROptions,
  amadeus_id: UserId
}

impl Handler {
  pub fn new(iopts: &IOptions, ropts: ROptions, amadeus: UserId) -> Handler {
    Handler {
      ioptions: iopts.clone(),
      roptions: ropts,
      amadeus_id: amadeus
    }
  }
}

lazy_static! {
  pub static ref BACKUP: Mutex<VecDeque<(MessageId, Message)>>
    = Mutex::new(VecDeque::with_capacity(64));
  pub static ref MUTED: Mutex<Vec<UserId>> = Mutex::new(Vec::new());
}

#[async_trait]
impl EventHandler for Handler {
  async fn ready(&self, ctx: Context, ready: Ready) {
    info!("Connected as {}", ready.user.name);
    let guild_id = GuildId( self.ioptions.guild );
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Ok(member) = guild.member(&ctx,ready.user.id).await {
        if let Ok(some_permissions) = member.permissions(&ctx).await {
          if some_permissions.administrator() {
            info!("Running with Administrator permissions");
          } else {
            warn!("Amadeus needs Administrator permissions");
          }
        }
      }
    }
    voice::rejoin_voice_channel(&ctx, &self.roptions).await;
    let threads_check = THREADS.load(Ordering::Relaxed);
    if !threads_check {
      gate::behavior::activate(&ctx, &self.ioptions).await;
      THREADS.store(true, Ordering::Relaxed);
    }
  }
  async fn resume(&self, _ctx : Context, _ : ResumedEvent) {
    info!("Resumed");
  }
  async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, member: Member) {
    let mut muted_lock = MUTED.lock().await;
    if muted_lock.contains(&member.user.id) {
      if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
        if let Ok(mut member) = guild.member(&ctx, member.user.id).await {
          if let Some(role) = guild.role_by_name("muted") {
            if !member.roles.contains(&role.id) {
              if let Err(why) = member.add_role(&ctx, role).await {
                error!("Failed to assign muted role {:?}", why);
              } else {
                let mut found_users = vec![];
                for (i, u) in muted_lock.iter().enumerate() {
                  if *u == member.user.id {
                    found_users.push(i);
                  }
                }
                for i in found_users {
                  muted_lock.remove(i);
                }
              }
            }
          }
        }
      }
    }
    if let Ok(channels) = guild_id.channels(&ctx).await {
      let ai_text = chain::generate_with_language(&ctx, &guild_id, false).await;
      if let Some((channel, _)) = channel_by_name(&ctx, &channels, "log").await {
        let user = &member.user; // .read().await;
        let title = format!("has joined here, {}", &ai_text);
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            let mut e = e
              .author(|a| a.icon_url(&user.face()).name(&user.name))
              .title(title);
            if let Some(ref joined_at) = member.joined_at {
              e = e.timestamp(joined_at);
            } e
        })).await {
          error!("Failed to log new user {:?}", why);
        }
      }
    }
  }
  async fn guild_member_removal(&self, ctx: Context, guild_id: GuildId, user: User, _: Option<Member>) {
    let _was_on_chat = points::clear_points(*guild_id.as_u64(), *user.id.as_u64()).await;
    if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
      if let Ok(member) = guild.member(&ctx, user.id).await {
        if let Some(role) = guild.role_by_name("muted") {
          if member.roles.contains(&role.id) {
            let mut muted_lock = MUTED.lock().await;
            if !muted_lock.contains(&member.user.id) {
              muted_lock.push(member.user.id);
            }
          }
        }
      }
    }
    if let Ok(channels) = guild_id.channels(&ctx).await {
      let ai_text = chain::generate_with_language(&ctx, &guild_id, false).await;
      if let Some((channel, _)) = channel_by_name(&ctx, &channels, "log").await {
        let title = format!("has left, {}", &ai_text);
        if let Err(why) = channel.send_message(&ctx, |m| m
          .embed(|e| {
            e.author(|a| a.icon_url(&user.face()).name(&user.name))
              .title(title)
              .timestamp(chrono::Utc::now().to_rfc3339())
            })).await {
          error!("Failed to log leaving user {:?}", why);
        }
      }
    }
  }
  async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId) {
    if RESTORE.load(Ordering::Relaxed) {
      let backup_deq = BACKUP.lock().await;
      // message is deleted already, impossible to get it using http context
      // if let Ok(msg) = ctx.http.get_message(*channel_id.as_u64(), *deleted_message_id.as_u64()).await {
      if !backup_deq.is_empty() {
        if let Some((_, msg)) = backup_deq.iter().find(|(id, _)| *id == deleted_message_id) {
          if msg.is_own(&ctx).await { // TODO: not sure whether we want to backup ALL
            if let Some(guild_id) = msg.guild_id {
              if let Ok(audit) = ctx.http.get_audit_logs( *guild_id.as_u64()
                                                        , Some( ActionMessage::Delete as u8 )
                                                        , None
                                                        , None
                                                        , Some(1)).await {
                // Here we just hope it's last in Audit log, very unsafe stuff
                if let Some(entry) = audit.entries.values().next() {
                  // that entry contains target_id: Option<u64> but nobody knows what's that
                  if let Ok(deleter) = ctx.http.get_user(*entry.user_id.as_u64()).await {
                    if !deleter.bot {
                      // message was removed by admin or by author
                      info!("{} or {} was trying to remove message", deleter.name, msg.author.name);
                      // log(&ctx, &guild_id, &log_text).await;
                      // But I don't allow it
                      for file in &msg.attachments {
                        if let Ok(bytes) = file.download().await {
                          let cow = AttachmentType::Bytes {
                            data: Cow::from(bytes),
                            filename: String::from(&file.filename)
                          };
                          if let Err(why) = channel_id.send_message(&ctx, |m| m.add_file(cow)).await {
                            error!("Failed to download and post attachment {:?}", why);
                          }
                        }
                      }
                      if !msg.content.is_empty() {
                        if let Err(why) = channel_id.send_message(&ctx, |m|
                            m.content(&msg.content)
                          ).await {
                          error!("Failed to post my message again, {:?}", why);
                        };
                      }
                      for embed in &msg.embeds {
                        if let Err(why) = channel_id.send_message(&ctx, |m| {
                          m.embed(|e| {
                            *e = CreateEmbed::from(embed.clone());
                            e })
                        }).await {
                          error!("Error reposting embed {:?}", why);
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        } else {
          warn!("Failed to get deleted message");
        }
      }
    }
  }
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.is_own(&ctx).await {
      // TODO: no such check for commands
      let blame_check = BLAME.load(Ordering::Relaxed);
      if !blame_check {
        if let Some(g) = msg.guild_id {
          if !WHITELIST_SERVERS.iter().any(|s| *s == *g.as_u64()) {
            if let Ok(guild) = g.to_partial_guild(&ctx).await {
              if let Ok(member) = guild.member(&ctx, &msg.author.id).await {
                if let Ok(some_permissions) = member.permissions(&ctx).await {
                  if !some_permissions.administrator() {
                    BLAME.store(true, Ordering::Relaxed);
                    for _ in 0..3 {
                      channel_message(&ctx, &msg, "GIVE ME ADMIN ROLE F!!CKERS!").await;
                      channel_message(&ctx, &msg, "OR I WILL BURN YOUR HOME!").await;
                      channel_message(&ctx, &msg, "I WILL EAT YOUR PETS!").await;
                      channel_message(&ctx, &msg, "DON'T MESS WITH ME!").await;
                      channel_message(&ctx, &msg, "GIVE ME ADMINISTRATOR OR DIE!").await;
                    }
                    BLAME.store(false, Ordering::Relaxed);
                  }
                }
              }
            }
          }
        }
        let mut backup_deq = BACKUP.lock().await;
        if backup_deq.len() == backup_deq.capacity() {
          backup_deq.pop_front();
        }
        backup_deq.push_back((msg.id, msg));
      }
    } else if msg.author.bot {
      // do nothing if that's whitelisted server
      if let Some(g) = msg.guild_id {
        if WHITELIST_SERVERS.iter().any(|s| *s == *g.as_u64()) {
          return;
        }
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
          r.limit(5)
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
        if (&embed.description).is_some()
          && embed.description.as_ref().unwrap().contains("DiscordAPIError") {
          not_stupid_zephyr = false
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
        if let Some(channel_name) = msg.channel_id.name(&ctx).await {
          if IGNORED.iter().any(|i| i == &channel_name) {
            return;
          }
        }
        if (&msg.mentions).iter().any(|u| u.bot) {
          if (&msg.mentions).iter().any(|u| u.bot && u.id == self.amadeus_id) {
            let amention1 = format!("<@{}>", self.amadeus_id);
            let amention2 = format!("<@!{}>", self.amadeus_id);
            if !msg.content.starts_with(&amention1)
            && !msg.content.starts_with(&amention2) {
              chain::response(&ctx, &msg).await;
            }
          }
        } else {
          points::add_points(*guild_id.as_u64(), *msg.author.id.as_u64(), 1).await;
          // TODO: move replay stuff in some function
          for file in &msg.attachments {
            if file.filename.ends_with("w3g") {
              let fname_apm = format!("{}_apm.png", &file.filename);
              if let Ok(bytes) = file.download().await {
                let mut fw3g = match File::create(&file.filename).await {
                  Ok(replay) => replay,
                  Err(why) => {
                    error!("Error creating file: {:?}", why);
                    channel_message(&ctx, &msg, "Error getting replay").await;
                    return;
                  }
                };
                if let Err(why) = fw3g.write_all(&bytes).await {
                  error!("Error writing to file: {:?}", why);
                  if let Err(why2) = fs::remove_file(&file.filename).await {
                    error!("Error removing file: {:?}", why2);
                  }
                  return;
                }
                let _ = fw3g.sync_data().await;
                let data_maybe = analyze(&file.filename).await;
                if let Err(why) = data_maybe {
                  error!("Corrupted replay file? {:?}", why);
                  if let Err(why2) = fs::remove_file(&file.filename).await {
                    error!("Error removing file: {:?}", why2);
                  }
                  return;
                }
                let (d, flds) = data_maybe.unwrap();
                let mut eb1 = CreateEmbed::default();
                let mut eb2 = CreateEmbed::default();
                let mut eb3 = CreateEmbed::default();
                let footer = format!("Uploaded by {}", msg.author.name);
                eb1.color(0xe535cc); eb2.color(0xe535cc); eb3.color(0xe535cc);
                eb1.title(&file.filename); eb2.title(&file.filename); eb3.title(&file.filename);
                eb1.description(&d); eb2.description("units"); eb3.description("apm");
                eb1.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
                eb2.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
                eb3.thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png");
                eb1.footer(|f| f.text(&footer));
                eb2.footer(|f| f.text(&footer));
                eb3.footer(|f| f.text(&footer));
                let mut max_apm = 0;
                if !flds.is_empty() {
                  let mut fields1 = vec![];
                  let mut fields2 = vec![];
                  let mut fields3 = vec![];
                  for (kk, vv, mut papm) in flds {
                    if vv.len() > 1 {
                      fields1.push((kk.clone(), vv[0].clone(), true));
                      fields2.push((kk.clone(), vv[1].clone(), true));
                    }
                    if !papm.len() > 1 {
                      // drop last value of apm, because it's "not full"
                      papm.truncate(papm.len() - 1);
                      let max = papm.iter().max().unwrap_or_else(|| &0);
                      max_apm = std::cmp::max(max_apm, *max);
                      fields3.push(
                        ( kk.clone()
                        , papm.into_iter().enumerate().map(|(i, x)| (i as f32, x as f64))
                        )
                      );
                    }
                  }
                  let mut apm_image : Option<String> = None;
                  if !fields3.is_empty() {
                    let (_, first_amp_list) = &fields3[0];
                    let len: f32 = first_amp_list.len() as f32 - 1_f32;
                    //TODO: I need different text color here...
                    { // because of Rc < > in BitMapBackend I need own scope here
                      let root_area = BitMapBackend::new(&fname_apm, (1024, 768)).into_drawing_area();
                      root_area.fill(&RGBColor(47, 49, 54)).unwrap(); //2f3136
                      let mut cc = ChartBuilder::on(&root_area)
                        .margin(5)
                        .set_all_label_area_size(50)
                        .build_cartesian_2d(0.0..len, 0.0..max_apm as f64)
                        .unwrap();
                      cc.configure_mesh()
                        .label_style(("monospace", 14).into_font().color(&RGBColor(150, 150, 150)))
                        .y_labels(10)
                        .axis_style(&RGBColor(80, 80, 80))
                        .draw().unwrap();
                      for (k, plx) in fields3 {
                        set! { red   = rand::thread_rng().gen_range(0, 255)
                             , green = rand::thread_rng().gen_range(0, 255)
                             , blue  = rand::thread_rng().gen_range(0, 255) };
                        let mut color = RGBColor(red, green, blue);
                        if red < 150 && green < 150 && blue < 150 {
                          set! { red2   = rand::thread_rng().gen_range(100, 255)
                               , green2 = rand::thread_rng().gen_range(100, 255)
                               , blue2  = rand::thread_rng().gen_range(100, 255) };
                          color = RGBColor(red2, green2, blue2);
                        }
                        cc.draw_series(LineSeries::new(plx, &color)).unwrap()
                          .label(&k)
                          .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
                      }
                      cc.configure_series_labels()
                        .border_style(&BLACK)
                        .label_font(("monospace", 17).into_font().color(&RGBColor(200, 200, 200)))
                        .draw().unwrap();
                    }
                    let amadeus_guild_id = GuildId( self.ioptions.amadeus_guild );
                    let amadeus_storage =
                      if let Ok(amadeus_channels) = amadeus_guild_id.channels(&ctx).await {
                        if let Some((ch, _)) = channel_by_name(&ctx, &amadeus_channels, "apm_pics").await {
                          Some(*ch)
                        } else { None }
                      } else { None };
                    if let Some(storage) = &amadeus_storage {
                      match storage.send_message(&ctx, |m|
                        m.add_file(AttachmentType::Path(std::path::Path::new(&fname_apm)))).await {
                        Ok(msg) => {
                          if !msg.attachments.is_empty() {
                            let img_attachment = &msg.attachments[0];
                            apm_image = Some(img_attachment.url.clone());
                          }
                        },
                        Err(why) => {
                          error!("Failed to download and post stream img {:?}", why);
                        }
                      };
                    }
                  }
                  eb1.fields(fields1);
                  eb2.fields(fields2);
                  if let Some(apm) = apm_image {
                    eb3.image(apm);
                  }
                }
                let embeds = vec![ eb1, eb3, eb2 ];
                if let Ok(mut bot_msg) = msg.channel_id.send_message(&ctx, |m| {
                                           m.embed(|e| { e.0 = embeds[0].0.clone(); e })
                                         }).await {
                  let mut page : usize = 0;
                  let left = ReactionType::Unicode(String::from("⬅️"));
                  let right = ReactionType::Unicode(String::from("➡️"));
                  let _ = bot_msg.react(&ctx, left).await;
                  let _ = bot_msg.react(&ctx, right).await;
                  loop {
                    if let Some(reaction) =
                      &bot_msg.await_reaction(&ctx)
                              .timeout(Duration::from_secs(360)).await {
                      let emoji = &reaction.as_inner_ref().emoji;
                      match emoji.as_data().as_str() {
                        "⬅️" => { 
                          if page != 0 {
                            page -= 1;
                          }
                        },
                        "➡️" => { 
                          if page != 2 {
                            page += 1;
                          }
                        },
                        _ => (),
                      }
                      if let Err(err) = bot_msg.edit(&ctx, |m|
                        m.embed(|mut e| {
                          e.0 = embeds[page].0.clone(); e
                        })
                      ).await {
                        error!("Shit happens {:?}", err);
                      }
                      let _ = reaction.as_inner_ref().delete(&ctx).await;
                    } else {
                      let _ = bot_msg.delete_reactions(&ctx).await;
                      break;
                    };
                  }
                } else {
                  error!("Failed to post replay analyze data");
                }
                if let Err(why1) = fs::remove_file(&fname_apm).await {
                  error!("Error removing apm png {:?}", why1);
                }
                if let Err(why2) = fs::remove_file(&file.filename).await {
                  error!("Error removing file: {:?}", why2);
                }
                return;
              }
            }
          }
          let is_admin =
            if let Some(member) = msg.member(&ctx.cache).await {
              if let Ok(permissions) = member.permissions(&ctx.cache).await {
                permissions.administrator()
              } else { false }
            } else {false };
          if !is_admin {
            gate::LAST_CHANNEL.store(*msg.channel_id.as_u64(), Ordering::Relaxed);
            // wakes up on any activity
            let rndx = rand::thread_rng().gen_range(0, 3);
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
                  let re_ib = Regex::new(r"<(.*?)>").unwrap();
                  let replaced = re_ib.replace_all(&activity, "");
                  if !replaced.is_empty() {
                    ctx.set_activity(Activity::playing(&replaced)).await;
                  }
                } else {
                  ctx.set_activity(Activity::playing(&activity)).await;
                }
                ctx.idle().await;
              }
            }
            let channel_name =
              if let Some(ch) = msg.channel(&ctx).await {
                ch.id().name(&ctx).await.unwrap_or_else(|| "".to_string())
              } else { "".to_string() };
            if AI_ALLOWED.iter().any(|c| c == &channel_name) {
              let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
              let rnd = rand::thread_rng().gen_range(0, activity_level);
              if rnd == 1 {
                chain::chat(&ctx, &msg).await;
              }
              let rnd2 : u16 = rand::thread_rng().gen_range(0, 2);
              if rnd2 == 1 {
                let mut rng = StdRng::from_entropy();
                let emoji = REACTIONS.choose(&mut rng).unwrap();
                let reaction = ReactionType::Custom {
                  animated: false,
                  id: EmojiId(emoji.id),
                  name: Some(emoji.name.clone())
                };

                if let Some(_ch) = msg.channel(&ctx).await {

                  if let Ok(guild) = guild_id.to_partial_guild(&ctx).await {
                    if let Ok(mut member) = guild.member(&ctx, msg.author.id).await {
                      if let Some(role) = guild.role_by_name("UNBLOCK AMADEUS") {

                        let normal_people_rnd : u16 = rand::thread_rng().gen_range(0, 9);
                        if (normal_people_rnd == 1 || member.roles.contains(&role.id))
                        && !WHITELIST_SERVERS.iter().any(|s| *s == *guild_id.as_u64())
                        && !WHITELIST.iter().any(|u| *u == *msg.author.id.as_u64()) {

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
                                let new_nick : String = format!("Hater {}", &msg.author.name);
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

                        let rnd3 = rand::thread_rng().gen_range(0, 9);
                        if rnd3 != 1 {
                          if let Err(why) = msg.delete_reactions(&ctx).await {
                            error!("Failed to remove all the reactions {:?}", why);
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
      if let Some(find_char_in_words) = OVERWATCH.iter().find(|c| {
        let regex = format!(r"(^|\W)((?i){}(?-i))($|\W)", c);
        let is_overwatch = Regex::new(&regex).unwrap();
        is_overwatch.is_match(&msg.content) })
      {
        let mut rng = StdRng::from_entropy();
        set! { ov_reply = OVERWATCH_REPLIES.choose(&mut rng).unwrap()
              , reply = format!("{} {}", ov_reply, find_char_in_words) };
        if let Err(why) = msg.channel_id.say(&ctx, reply).await {
          error!("Error sending overwatch reply: {:?}", why);
        }
      } else {
        let regex_no_u = Regex::new(r"(^|\W)((?i)no u(?-i))($|\W)").unwrap();
        if regex_no_u.is_match(&msg.content) {
          let rnd = rand::thread_rng().gen_range(0, 2);
          if rnd == 1 {
            if let Err(why) = msg.channel_id.say(&ctx, "No u").await {
              error!("Error sending no u reply: {:?}", why);
            }
          }
        }
      }
      /* TODO: not sure for users messages
      let mut backup_deq = BACKUP.lock().await;
      if backup_deq.len() == backup_deq.capacity() {
        backup_deq.pop_front();
      }
      backup_deq.push_back((msg.id, msg));
      */
    }
  }
  async fn guild_ban_addition(&self, _ctx: Context, _guild_id: GuildId, _banned_user: User) {}
  async fn guild_ban_removal(&self, _ctx: Context, _guild_id: GuildId, _unbanned_user: User) {}
}

use crate::common::msg::channel_message;

use serenity::{
  prelude::*,
  builder::*,
  model::channel::{ Attachment
                  , Message
                  , ReactionType },
  builder::CreateEmbed
};

use std::{
  time::Duration,
  borrow::Borrow
};

use async_std::{ fs::File, fs
               , prelude::* };

use w3rs::{
  event::{ Event, GameEvent },
  action::Action,
  game::Game
};

fn non_noisy(event: &&GameEvent) -> bool {
  match &event.event {
    // avoid noisy actions
    Event::Action {
      action,
      selection: _,
    } => !matches!(
      action,
      Action::Move(_)
        | Action::SetRallyPoint(_)
        | Action::RightClick { at: _, target: _ }
        | Action::Attack { at: _, target: _ }
    ),
    _ => true,
  }
}

pub async fn analyze_with_w3rs(path: &str) -> Vec<String> {
  let game = Game::parse(path);
  let mut actions = vec![];
  for event in game.events().iter().filter(non_noisy).collect::<Vec<&GameEvent>>() {
    if let Some(player) = game.players.iter().find(|p| p.id == event.player_id) {
      let player_name = &player.name;
      match &event.event {
        Event::ChatMsg { addressee, message } =>
        actions.push( format!("{player_name}: {addressee}: {message}") ),
        Event::Action { selection, action } => {
          actions.push( format!("{player_name}: {:?} {action}", selection) )
        }
      };
    }
  }
  actions
}

pub async fn rep_embed( ctx: &Context
                      , msg: &Message
                      , file: &Attachment ) -> anyhow::Result<()> {
  info!("Downloading replay");
  if let Ok(bytes) = file.download().await {
    let mut fw3g = match File::create(&file.filename).await {
      Ok(replay) => replay,
      Err(why) => {
        channel_message(ctx, msg, "Error getting replay").await;
        return Err(anyhow!("Error creating file: {why}"));
      }
    };
    if let Err(why) = fw3g.write_all(&bytes).await {
      if let Err(why2) = fs::remove_file(&file.filename).await {
        error!("Error removing file: {why2}");
      }
      return Err(anyhow!("Error writing to file: {why}"));
    }
    let _ = fw3g.sync_data().await;
    info!("Parsing replay");
    let data_maybe = analyze_with_w3rs(&file.filename).await;
    if let Err(why2) = fs::remove_file(&file.filename).await {
      error!("Error removing file: {why2}");
    }
    if data_maybe.is_empty() {
      error!("can't parse replay {}", &file.filename);
      return Ok(());
    }
    let mut embeds = vec![];

    let footer = format!("Uploaded by {}", msg.author.name);

    for (i, chunk) in data_maybe.chunks(20).enumerate() {
      let embed = CreateEmbed::new()
        .title(&format!("{} #{}", &file.filename, i + 1))
        .description(chunk.join("\n"))
        .colour((180,40,200))
        .footer(CreateEmbedFooter::new(&footer));
      embeds.push(embed);
    }

    if !embeds.is_empty() {
      let mut page = 0;
      let mut bot_msg = msg.channel_id.send_message(ctx, CreateMessage::new().embed(
        embeds[page].clone()
      )).await?;
      if embeds.len() > 1 {
        let left = ReactionType::Unicode(String::from("⬅️"));
        let right = ReactionType::Unicode(String::from("➡️"));
        let _ = bot_msg.react(ctx, left).await;
        let _ = bot_msg.react(ctx, right).await;
        loop {
          let collector = bot_msg.reaction_collector(&ctx.shard)
                                 .timeout(Duration::from_secs(120))
                                 .author_id(msg.author.id);
          if let Some(reaction) = collector.collect_single().await {
            let emoji = &reaction.as_inner_ref().emoji;
            match emoji.as_data().borrow() {
              "⬅️" => { 
                if page != 0 {
                  page -= 1;
                }
              },
              "➡️" => { 
                if page != embeds.len() - 1 {
                  page += 1;
                }
              },
              _ => (),
            }
            bot_msg.edit(ctx, EditMessage::default().embed(
              embeds[page].clone()
            )).await?;
            let _ = reaction.as_inner_ref().delete(ctx).await;
          } else {
            let _ = bot_msg.delete_reactions(ctx).await;
            break;
          };
        }
      }
    }

    if let Err(why2) = fs::remove_file(&file.filename).await {
      error!("Error removing file: {why2}");
    }
  }
  Ok(())
}

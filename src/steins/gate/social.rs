use crate::{
  common::help::channel::channel_by_name,
  steins::ai::{ chain, bert },
  commands::pad::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    id::ChannelId,
    channel::GuildChannel,
  }
};

use std::{
  collections::HashMap,
  sync::atomic::Ordering,
  time
};

use rand::Rng;

/* every 30 minutes */
static POLL_PERIOD_SECONDS: u64 = 30 * 60;
/* every ~2 hours, depends on ACTIVITY_LEVEL */
static PASSED_FOR_CONVERSATION: u32 = 2 * 60 * 60 / POLL_PERIOD_SECONDS as u32;

pub async fn activate_social_skils(
                     ctx:       &Context
                   , channels:  &HashMap<ChannelId, GuildChannel> ) {
  if let Some((channel, _)) = channel_by_name(&ctx, &channels, "main").await {
    set!{ ch_deref  = *channel
        , ctx_clone = ctx.clone() };
    tokio::spawn(async move {
      loop {
        let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
        let rndx = rand::thread_rng().gen_range(0, activity_level);
        if rndx == 1 {
          let ai_text = chain::generate_with_language(&ctx_clone, false).await;
          if let Err(why) = ch_deref.send_message(&ctx_clone, |m| {
            m.content(ai_text)
          }).await {
            error!("Failed to post periodic message {:?}", why);
          }
        } else {
          // clean up old bert model conversation id-s
          let mut k_to_del: Vec<u64> = Vec::new();
          let mut chat_context = bert::CHAT_CONTEXT.lock().await;
          for (k, (_, passed_time, _)) in chat_context.iter_mut() {
            if *passed_time < PASSED_FOR_CONVERSATION {
              *passed_time += 1;
            } else {
              k_to_del.push(*k);
            }
          }
          for ktd in k_to_del {
            trace!("removing conversation {} with timeout", ktd);
            chat_context.remove(&ktd);
          }
          update_current_season().await;
        }
        tokio::time::delay_for(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
      }
    });
  }
  if let Some((channel, _)) = channel_by_name(&ctx, &channels, "💬главный-зал💬").await {
    set!{ ch_deref  = *channel
        , ctx_clone = ctx.clone() };
    tokio::spawn(async move {
      loop {
        let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
        let rndx = rand::thread_rng().gen_range(0, activity_level);
        if rndx == 1 {
          let ai_text = chain::generate_with_language(&ctx_clone, true).await;
          if let Err(why) = ch_deref.send_message(&ctx_clone, |m| {
            m.content(ai_text)
          }).await {
            error!("Failed to post periodic message {:?}", why);
          }
        }
        tokio::time::delay_for(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
      }
    });
  }
}

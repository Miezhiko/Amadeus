use crate::{
  common::help::channel::channel_by_name,
  stains::ai::chain,
  commands::pad::update_current_season
};

use serenity::{
  prelude::*,
  model::{
    id::GuildId, id::ChannelId,
    channel::GuildChannel,
  }
};

use std::{
  collections::HashMap,
  sync::atomic::Ordering,
  time
};

use rand::Rng;

pub async fn activate_social_skils(
                     ctx:       &Context
                   , channels:  &HashMap<ChannelId, GuildChannel>
                   , guild_id:  GuildId ) {
  if let Some((channel, _)) = channel_by_name(&ctx, &channels, "main").await {
    set!{ ch_deref  = *channel
        , ctx_clone = ctx.clone() };
    tokio::spawn(async move {
      loop {
        let activity_level = chain::ACTIVITY_LEVEL.load(Ordering::Relaxed);
        let rndx = rand::thread_rng().gen_range(0, activity_level);
        if rndx == 1 {
          let ai_text = chain::generate_english_or_russian(&ctx_clone, &guild_id).await;
          if let Err(why) = ch_deref.send_message(&ctx_clone, |m| {
            m.content(ai_text)
          }).await {
            error!("Failed to post periodic message {:?}", why);
          }
        }
        update_current_season().await;
        /* every 30 minutes */
        tokio::time::delay_for(time::Duration::from_secs(30*60)).await;
      }
    });
  }
}

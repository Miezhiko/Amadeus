use crate::{
  common::constants::MAIN_CHANNEL,
  steins::ai::{ cache, chain }
};

use serenity::{
  prelude::*,
  builder::CreateMessage,
  model::id::ChannelId
};

use std::{
  sync::atomic::Ordering,
  time,
  sync::Arc
};

use rand::Rng;

/* every 2 hours */
static POLL_PERIOD_SECONDS: u64 = 2 * 60 * 60;

pub async fn activate_social_skils(ctx: &Arc<Context>) {
  let ctx_clone = Arc::clone(ctx);
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
      {
        let activity_level = cache::ACTIVITY_LEVEL.load(Ordering::Relaxed) + 20;
        let rndx = if activity_level > 0
               { rand::thread_rng().gen_range(0..activity_level) }
          else { 666 };
        if rndx < 2 {
          let (chanz, ru) = match rndx {
            0 => { (MAIN_CHANNEL, true) },
            _ => { ( ChannelId::new( 1052777234454294569 ), false) }
          };
          let ai_text = chain::generate_with_language(&ctx_clone, ru).await;
          if let Err(why) = chanz.send_message(&ctx_clone, CreateMessage::new()
            .content(ai_text)
          ).await {
            error!("Failed to post periodic message {why}");
          }
        }
      }
    }
  });
}

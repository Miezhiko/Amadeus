use crate::{
  common::{
    system,
    constants::{
      MAIN_CHANNEL,
      LOG_CHANNEL
    }
  },
  steins::ai::{ cache, chain, bert },
  commands::pad::update_current_season
};

use serenity::prelude::*;

use std::{
  sync::atomic::Ordering,
  time,
  sync::Arc
};

use rand::Rng;

/* every 30 minutes */
static POLL_PERIOD_SECONDS: u64 = 30 * 60;
/* every ~2 hours, depends on ACTIVITY_LEVEL */
static PASSED_FOR_CONVERSATION: u32 = 2 * 60 * 60 / POLL_PERIOD_SECONDS as u32;

pub async fn activate_social_skils(ctx: &Arc<Context>) {

  let ctx_clone = Arc::clone(&ctx);
  tokio::spawn(async move {
    loop {
      let activity_level = cache::ACTIVITY_LEVEL.load(Ordering::Relaxed) + 10;
      let rndx = rand::thread_rng().gen_range(0..activity_level);
      if rndx == 1 {
        let ai_text = chain::generate_with_language(&ctx_clone, false).await;
        if let Err(why) = MAIN_CHANNEL.send_message(&ctx_clone, |m| {
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
        update_current_season(&ctx_clone).await;

        // memory check!
        if let Ok(mem_mb) = system::get_memory_mb().await {
          // USE 24 GB RAM LIMIT FOR NOW
          if mem_mb > 1024.0 * 24.0 {
            if let Err(why) = system::upgrade_amadeus(&ctx_clone, &LOG_CHANNEL).await {
              error!("Failed to run upgrade {:?}", why);
            }
          }
        }

      }
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
    }
  });

}

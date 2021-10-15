use crate::{
  common::{ system
          , constants::MODERATION },
  steins::ai::{ bert, reinit },
  commands::w3c::update_current_season
};

use serenity::prelude::*;

use chrono::{ Duration, Utc };

use std::{
  time,
  sync::Arc
};

/* every 30 minutes */
static POLL_PERIOD_SECONDS: u64 = 30 * 60;
/* every ~2 hours, depends on ACTIVITY_LEVEL */
static PASSED_FOR_CONVERSATION: u32 = 2 * 60 * 60 / POLL_PERIOD_SECONDS as u32;

pub async fn activate_system_tracker(ctx: &Arc<Context>, lsm: bool) {
  let ctx_clone = Arc::clone(&ctx);
  tokio::spawn(async move {
    loop {
      {
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

        if ! lsm {
          let mut convmodel_used = bert::CONVMODEL_USED.lock().await;
          if let Some(conv_model_used_time) = &*convmodel_used {
            let nao = Utc::now();
            let since_last_update: Duration = nao - *conv_model_used_time;
            if since_last_update > Duration::minutes(10) {
              let mut convmodel = bert::CONVMODEL.lock().await;
              *convmodel = None;
              *convmodel_used = None;
            }
          }
        }

        // memory check!
        if let Ok(mem_mb) = system::stats::get_memory_mb().await {
          // USE 24 GB RAM LIMIT FOR NOW
          if mem_mb > 1024.0 * 24.0 {
            if let Err(why) = system::upgrade::upgrade_amadeus(&ctx_clone, &MODERATION).await {
              error!("Failed to run upgrade {:?}", why);
            }
          } else if mem_mb > 1024.0 * 13.0 {
            // soft reset on 13 gb
            reinit().await;
          }
        }
      }
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
    }
  });
}

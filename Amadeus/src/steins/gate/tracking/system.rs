use crate::{
  types::serenity::IContext,
  common::{ system, salieri::SALIERI
          , constants::GITHUB_PRS },
  steins::ai::reinit
};

use serenity::prelude::*;

use std::{
  time, sync::Arc
};

/* every 30 minutes */
static POLL_PERIOD_SECONDS: u64 = 30 * 60;

pub async fn activate_system_tracker(ctx: &Arc<Context>) {
  let ctx_clone = Arc::clone(ctx);
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
      { // this scope is needed for async locks!
        // clean up old bert model conversation id-s
        let salieri_lock = SALIERI.lock().await;
        if let Some(salieri) = &*salieri_lock {
          if let Err(why) = salieri.send_task(
                              mozart::cache::CONTEXT_CLEAR::new()
                            ).await {
            error!("failed to clear context {why}");
          }

          let lsm = {
            let data = ctx_clone.data.read().await;
            if let Some(icontext) = data.get::<IContext>() {
              icontext.lazy_static_models
            } else { false }
          };
          if !lsm {
            if let Err(why) = salieri.send_task(
                                mozart::cache::MODELS_REINIT::new()
                              ).await {
              error!("failed to reinit models {why}");
            }
          }
        }

        // memory check!
        if let Ok(mem_mb) = system::stats::get_memory_mb().await {
          // USE 24 GB RAM LIMIT FOR NOW
          if mem_mb > 1024.0 * 24.0 {
            if let Err(why) = system::upgrade::upgrade_amadeus(&ctx_clone, &GITHUB_PRS).await {
              error!("Failed to run upgrade {:?}", why);
            }
          } else if mem_mb > 1024.0 * 13.0 {
            // soft reset on 13 gb
            reinit().await;
          }
        }
      }
    }
  });
}

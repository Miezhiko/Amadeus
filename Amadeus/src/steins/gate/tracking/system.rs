use crate::{
  types::serenity::IContext,
  common::{ system
          , constants::GITHUB_PRS },
  steins::ai::{ bert, reinit }
};

use mozart::bert::chat::{ CHAT_CONTEXT, CONVMODEL_USED, CONVMODEL };

use serenity::prelude::*;

use chrono::{ Duration, Utc };

use std::{ time, sync::Arc };

/* every 30 minutes */
static POLL_PERIOD_SECONDS: u64 = 30 * 60;

pub async fn activate_system_tracker(ctx: &Arc<Context>) {
  let ctx_clone = Arc::clone(ctx);
  tokio::spawn(async move {
    loop {
      { // this scope is needed for async locks!
        // clean up old bert model conversation id-s
        let mut chat_context = CHAT_CONTEXT.lock().await;
        chat_context.clear();

        let lsm = {
          let data = ctx_clone.data.read().await;
          if let Some(icontext) = data.get::<IContext>() {
            icontext.lazy_static_models
          } else { false }
        };
        if !lsm {
          let mut convmodel_used = CONVMODEL_USED.lock().await;
          if let Some(conv_model_used_time) = &*convmodel_used {
            let nao = Utc::now();
            let since_last_use: Duration = nao - *conv_model_used_time;
            if since_last_use > Duration::minutes(10) {
              let mut convmodel = CONVMODEL.lock().await;
              *convmodel = None;
              *convmodel_used = None;
            }
          }
          // don't free ENRU model if CONV model is loaded.
          if convmodel_used.is_none() {
            let mut enru_used = bert::ENRUMODEL_USED.lock().await;
            if let Some(enru_model_used_time) = &*enru_used {
              let nao = Utc::now();
              let since_last_use: Duration = nao - *enru_model_used_time;
              if since_last_use > Duration::minutes(30) {
                let mut enrumodel = bert::ENRUMODEL.lock().await;
                *enrumodel = None;
                *enru_used = None;
              }
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
      tokio::time::sleep(time::Duration::from_secs(POLL_PERIOD_SECONDS)).await;
    }
  });
}

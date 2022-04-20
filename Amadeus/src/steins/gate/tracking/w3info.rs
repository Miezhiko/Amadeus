use crate::{
  collections::team::DISCORDS,
  commands::warcraft
};

use serenity::prelude::*;

use std::{
  time,
  sync::Arc
};

use chrono::{ Duration, DateTime, Utc };

#[allow(clippy::single_element_loop)]
pub async fn activate_w3info_tracking(ctx: &Arc<Context> ) {
  let ctx_clone = Arc::clone(ctx);
  tokio::spawn(async move {
    loop {
      let today: DateTime<Utc> = Utc::now();
      for (_, df) in DISCORDS.iter() {
        if let Some(chan) = df.events {
          if let Err(why) =
            warcraft::tour_internal( &ctx_clone
                                   , &chan, today
                                   , false, false
                                   ).await {
            error!("Failed to post today tour events, {why}");
          }
          if let Err(why) =
            warcraft::tour_internal( &ctx_clone
                                   , &chan, today + Duration::days(1)
                                   , false, false
                                   ).await {
            error!("Failed to post tomorrow tour events, {why}");
          }
        }
      }
      // check every 12 hours
      tokio::time::sleep(time::Duration::from_secs(60*60*12)).await;
    }
  });
}

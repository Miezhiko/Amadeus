use crate::commands::warcraft;

use serenity::{
  prelude::*,
  model::id::ChannelId
};

use std::{
  time,
  sync::Arc
};

use chrono::{ Duration, DateTime, Utc };

pub async fn activate_w3info_tracking(ctx: &Arc<Context> ) {

  // TODO: move to dhall config
  let thread_channels = vec![ ChannelId ( 635912696675565608 ) // Fake News
                            , ChannelId ( 742643130096156672 ) // Lilualia
                            , ChannelId ( 781139361423949884 ) // Korchma
                            ];

  let ctx_clone = Arc::clone(&ctx);

  tokio::spawn(async move {
    loop {
      let today: DateTime<Utc> = Utc::now();
      for chan in &thread_channels {
        if let Err(why) =
          warcraft::tour_internal( &ctx_clone
                                  , &chan, today
                                  , false, false
                                  ).await {
          error!("Failed to post today tour events {:?}", why);
        }
        if let Err(why) =
          warcraft::tour_internal( &ctx_clone
                                  , &chan, today + Duration::days(1)
                                  , false, false
                                  ).await {
          error!("Failed to post tomorrow tour events {:?}", why);
        }
      }
      // check every 12 hours
      tokio::time::delay_for(time::Duration::from_secs(60*60*12)).await;
    }
  });
}

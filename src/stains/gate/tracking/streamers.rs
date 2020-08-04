use crate::{
  types::{
    options::IOptions,
    tracking::TrackingGame,
    twitch::Twitch,
    goodgame::GoodGameData
  },
  collections::team::teammates,
  common::help::channel::channel_by_name
};

use serenity::{
  prelude::*,
  model::{
    id::ChannelId,
    channel::GuildChannel
  }
};

use std::{
  collections::HashMap,
  time
};

lazy_static! {
  pub static ref STREAMS: Mutex<HashMap<u64, TrackingGame>>
    = Mutex::new(HashMap::new());
}

pub async fn activate_streamers_tracking(
                     ctx:       &Context
                   , channels:  &HashMap<ChannelId, GuildChannel>
                   , options:   &IOptions ) {

  if let Some((shannel, _)) = channel_by_name(&ctx, &channels, "live-streams").await {

    // Delete streams from live-streams channel (if some)
    // TODO: change 1 to 50 or something when it will work for long enough time
    if let Ok(vec_msg) = shannel.messages(&ctx, |g| g.limit(5)).await {
      let mut vec_id = Vec::new();
      for message in vec_msg {
        for embed in message.embeds {
          if let Some(title) = embed.title {
            if title != "FINISHED" {
              vec_id.push(message.id);
              break;
            }
          }
        }
      }
      if !vec_id.is_empty() {
        match shannel.delete_messages(&ctx, vec_id.as_slice()).await {
          Ok(nothing)  => nothing,
          Err(err) => warn!("Failed to clean live messages {}", err),
        };
      }
    }

    set!{ sh_deref      = *shannel
        , ctx_clone     = ctx.clone()
        , options_clone = options.clone() };
    tokio::spawn(async move {
      let mut streams_lock = STREAMS.lock().await;
      loop {
        let mut k_to_del : Vec<u64> = Vec::new();
        for (k, track) in streams_lock.iter_mut() {
          if track.passed_time < (60 * 24) {
            // for real it's 60 + some time for processing
            track.passed_time += 1;
          } else {
            k_to_del.push(*k);
          }
        }
        for ktd in k_to_del {
          warn!("stream {} out with timeout", ktd);
          streams_lock.remove(&ktd);
        }
        info!("streams check");
        for playa in teammates() {
          if let Ok(user) = ctx_clone.http.get_user(playa.discord).await {
            setm!{ twitch_live        = false
                  , additional_fields  = Vec::new()
                  , title              = String::new()
                  , image              = None
                  , em_url             = None };
            if playa.streams.is_some() {
              let streams = playa.streams.clone().unwrap();
              if streams.twitch.is_some() {
                let client = reqwest::Client::new();
                let getq = format!("https://api.twitch.tv/helix/streams?user_login={}", streams.twitch.unwrap().as_str());
                if let Ok(res) = client
                  .get(getq.as_str())
                  .header("Authorization", options_clone.twitch_oauth.clone())
                  .header("Client-ID", options_clone.twitch_client_id.clone())
                  .send().await {
                  match res.json::<Twitch>().await {
                    Ok(t) => {
                      if !t.data.is_empty() {
                        let twd = &t.data[0];
                        let url = format!("https://www.twitch.tv/{}", twd.user_name);
                        let pic = twd.thumbnail_url.replace("{width}", "800")
                                                    .replace("{height}", "450");
                        if twd.type_string == "live" {
                          let viewers = format!("viewers: {}", twd.viewer_count);
                          additional_fields.push(("Live on twitch", viewers, true));
                          title       = twd.title.clone();
                          image       = Some(pic);
                          em_url      = Some(url);
                          twitch_live = true;
                        }
                      }
                    }, Err(why) => {
                      error!("Failed to parse twitch structs\nrequest: {}\nerror: {:?}", getq.as_str(), why);
                    }
                  }
                }
              }
              if streams.ggru.is_some() {
                let ggru = streams.ggru.clone().unwrap();
                let ggru_link = format!("http://api2.goodgame.ru/v2/streams/{}", ggru.as_str());
                if let Ok(gg) = reqwest::get(ggru_link.as_str()).await {
                  match gg.json::<GoodGameData>().await {
                    Ok(ggdata) => {
                      if ggdata.status == "Live" {
                        let url = format!("https://goodgame.ru/channel/{}", ggru.as_str());
                        if twitch_live {
                          let titurl =
                            format!("{}\n{}", ggdata.channel.title.as_str(), url);
                          additional_fields.push(("Live on ggru", titurl, true));
                        } else {
                          let viewers = format!("viewers: {}\nin chat: {}", ggdata.viewers, ggdata.users_in_chat);
                          additional_fields.push(("Live on ggru", viewers, true));
                          title  = ggdata.channel.title.clone();
                          image  = Some(ggdata.channel.thumb.clone());
                          em_url = Some(url);
                        }
                      }
                    }, Err(why) => {
                      error!("Failed to parse good game structs {:?}", why);
                    }
                  };
                }
              }
            }
            if !additional_fields.is_empty() {
              if let Some(track) = streams_lock.get(&playa.discord) {
                if let Ok(mut msg) = ctx_clone.http.get_message(*sh_deref.as_u64(), track.tracking_msg_id).await {
                  let footer = format!("Passed: {} min", track.passed_time);
                  let mut fields = Vec::new();
                  let mut img = None;
                  let mut url = None;
                  if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                    for f in msg.embeds[0].fields.clone() {
                      fields.push((f.name, f.value, f.inline));
                    }
                    img = msg.embeds[0].image.clone();
                    url = msg.embeds[0].url.clone();
                  };
                  let is_now_live = format!("{} is now live!", user.name.as_str());
                  if let Err(why) = msg.edit(&ctx_clone, |m| m
                    .embed(|e|  {
                      let mut e = e
                        .title(title)
                        .author(|a| a.icon_url(&user.face()).name(is_now_live.as_str()))
                        .footer(|f| f.text(footer));
                      if !fields.is_empty() {
                        e = e.fields(fields);
                      }
                      if let Some(some_img) = img {
                        e = e.image(some_img.url);
                      }
                      if let Some(some_url) = url {
                        e = e.url(some_url);
                      }
                      e
                    }
                  )).await {
                    error!("Failed to edit stream msg {:?}", why);
                  }
                }
              } else {
                let is_now_live = format!("{} started stream!", user.name.as_str());
                match sh_deref.send_message(&ctx_clone, |m| m
                  .embed(|e| {
                    let mut e = e
                      .title(title)
                      .author(|a| a.icon_url(&user.face()).name(is_now_live.as_str()));
                    if !additional_fields.is_empty() {
                      e = e.fields(additional_fields);
                    }
                    if let Some(some_image) = image {
                      e = e.image(some_image);
                    }
                    if let Some(some_url) = em_url {
                      e = e.url(some_url);
                    }
                    e
                  }
                )).await {
                  Ok(msg_id) => {
                    streams_lock.insert(playa.discord, TrackingGame {
                      tracking_msg_id: *msg_id.id.as_u64(),
                      passed_time: 0,
                      still_live: true,
                      player: playa }
                    );
                  },
                  Err(why) => {
                    error!("Failed to post live match {:?}", why);
                  }
                }
              }
            } else if let Some(track) = streams_lock.get(&playa.discord) {
              if let Ok(mut msg) = ctx_clone.http.get_message(*sh_deref.as_u64(), track.tracking_msg_id).await {
                let footer = format!("Passed: {} min", track.passed_time);
                let mut fields = Vec::new();
                let mut img = None;
                let mut url = None;
                if !msg.embeds.is_empty() && !msg.embeds[0].fields.is_empty() {
                  for f in msg.embeds[0].fields.clone() {
                    fields.push((f.name, f.value, f.inline));
                  }
                  img = msg.embeds[0].image.clone();
                  url = msg.embeds[0].url.clone();
                };
                if let Err(why) = msg.edit(&ctx_clone, |m| m
                  .embed(|e|  {
                    let mut e = e
                      .title("FINISHED")
                      .author(|a| a.icon_url(&user.face()).name(&user.name))
                      .footer(|f| f.text(footer));
                    if !fields.is_empty() {
                      e = e.fields(fields);
                    }
                    if let Some(some_img) = img {
                      e = e.image(some_img.url);
                    }
                    if let Some(some_url) = url {
                      e = e.url(some_url);
                    }
                    e
                  }
                )).await {
                  error!("Failed to edit stream msg {:?}", why);
                }
              }
              streams_lock.remove(&playa.discord);
            }
          }
          tokio::time::delay_for(time::Duration::from_millis(100)).await;
        }
        tokio::time::delay_for(time::Duration::from_secs(60)).await;
      }
    });
  }
}

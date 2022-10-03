use serenity::{
  prelude::*,
  builder::{ GetMessages, CreateMessage, CreateEmbed, EditMessage },
  model::{ channel::*
         , id::ChannelId },
  framework::standard::{ CommandResult
                       , macros::command }
};

use std::io::BufReader;
use tokio::task;

use chrono::{ prelude::*
            , Duration
            , Utc };

pub async fn tour_internal( ctx: &Context
                          , channel_id: ChannelId
                          , on: DateTime<Utc>
                          , passed_check: bool
                          , report_no_events: bool
                          ) -> CommandResult {

  let maybe_reader = task::spawn_blocking(move || {
    if let Ok(res) = reqwest::blocking::get("https://warcraft3.info/ical-events") {
      let buf = BufReader::new(res);
     Some(ical::IcalParser::new(buf))
    } else {
      None
    }
  }).await?;

  if let Some(reader) = maybe_reader {
    set! { str_date_now = on.format("%Y%m%d").to_string()
         , str_time_now = on.format("%H%M").to_string() };

    let mut eventos: Vec<(String, String, bool)> = Vec::new();

    set!{ utc      = chrono::Utc::now()
        , cet_time = utc.with_timezone(&chrono_tz::CET).time()
        , msk_time = utc.with_timezone(&chrono_tz::Europe::Moscow).time()
        , h_offset = msk_time.hour() - cet_time.hour() };

    for line in reader {
      match line {
        Ok(l) => {
          for e in l.events {
            setm!{ is_today = false
                 , tvstr = String::new()
                 , evstr = String::new() };

            for ep in e.properties {
              if ep.name == "DTSTART" {
                if let Some(val) = ep.value {
                  if val.len() >= 8 {
                    let str_date = &val[..8];
                    let not_passed = if passed_check {
                      if let Ok(local_utc_time) = str_time_now.parse::<i32>() {
                        let str_hour_mins = &val[9..13];
                        if let Ok(event_hours_mins) = str_hour_mins.parse::<i32>() {
                          local_utc_time < event_hours_mins
                        } else {true }
                      } else { true }
                    } else { true };

                    if str_date_now == str_date && not_passed {
                      is_today = true;
                      if val.len() >= 14 {
                        set! { str_hour = &val[9..11]
                             , str_min  = &val[11..13] };
                        let msk =
                          if let Ok(str_int) = str_hour.parse::<u32>() {
                            let mut msk_h = str_int + h_offset;
                            if msk_h >= 24 {
                               msk_h -= 24;
                            }
                            format!(" ({msk_h}:{str_min} MSK)")
                          } else { String::from("") };
                        tvstr = format!("• {str_hour}:{str_min} CET {msk}");
                      }
                    }
                  }
                }
              } else if is_today {
                if ep.name == "SUMMARY" {
                  if let Some(val) = &ep.value {
                    evstr = val.to_string();
                  }
                }
                if ep.name == "DESCRIPTION" {
                  if let Some(val2) = &ep.value {
                    evstr = format!("{evstr}\n<{val2}>");
                  }
                }
              }
            }
            if is_today && !evstr.is_empty() {
              eventos.push((tvstr, evstr, false));
            }
          }
        },
        Err(e) => error!("Failed to parse calendar line {e}")
      }
    }

    if !eventos.is_empty() {
      set!{ date_str_x = on.format("%e-%b (%A)").to_string()
          , title = format!("Events on {date_str_x}") };

      // So we have title now, let check if it's posted already or not
      // In case if that was posted, check if we need to update it
      // Then finally update if there is new information
      let mut do_nothing = false;
      let mut post_to_edit = None;
      if !passed_check && !report_no_events {
        if let Ok(vec_msg) = channel_id.messages(&ctx, GetMessages::default().limit(10)).await {
          for message in vec_msg {
            if message.is_own(ctx) {
              for embed in message.embeds {
                if let Some(e_title) = embed.title {
                  if title == e_title {
                    if embed.fields.len() == eventos.len() {
                      do_nothing = true;
                      for (i, (_,c,_)) in eventos.iter().enumerate() {
                        let msg_content = &embed.fields[i].value;
                        if c != msg_content {
                          do_nothing = false;
                        }
                      }
                    }
                    if !do_nothing {
                      post_to_edit = Some( message.id );
                    }
                    break;
                  }
                }
              }
            }
          }
        }
      }
      if let Some(msg_id) = post_to_edit {
        if let Ok(mut msg) = ctx.http.get_message( channel_id
                                                 , msg_id ).await {
          let embed = CreateEmbed::new()
            .title(title.as_str())
            .thumbnail("https://upload.wikimedia.org/wikipedia/en/4/4f/Warcraft_III_Reforged_Logo.png")
            .fields(eventos)
            .colour((255, 192, 203));
          if let Err(why) = msg.edit(&ctx, EditMessage::default()
            .embed(embed)).await {
            error!("Error editing w3info event: {why}");
          }
        }
      } else if !do_nothing {
        if let Err(why) = channel_id.send_message(&ctx, CreateMessage::new()
          .embed(CreateEmbed::new()
            .title(title.as_str())
            .thumbnail("https://upload.wikimedia.org/wikipedia/en/4/4f/Warcraft_III_Reforged_Logo.png")
            .fields(eventos)
            .colour((240, 160, 203)))).await {
          error!("Error sending w3info events: {why}");
        }
      }
    } else if report_no_events {
      if let Err(why) = channel_id.send_message(&ctx, CreateMessage::new()
        .content("I am sorry but I can't find anything at the momenet")
      ).await {
        error!("Error sending w3info error: {why}");
      }
    }
  }
  Ok(())
}

pub async fn tour(ctx: &Context, msg: &Message, on: DateTime<Utc>) -> CommandResult {
  tour_internal(ctx, msg.channel_id, on, false, true).await
}

#[command]
#[aliases(вчера)]
#[description("display yesterday events from w3info")]
pub async fn yesterday(ctx: &Context, msg: &Message) -> CommandResult {
  let yesterday: DateTime<Utc> = Utc::now() - Duration::days(1); 
  tour(ctx, msg, yesterday).await?;
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[aliases(сегодня)]
#[description("display today events from w3info")]
pub async fn today(ctx: &Context, msg: &Message) -> CommandResult {
  let today: DateTime<Utc> = Utc::now(); 
  tour_internal(ctx, msg.channel_id, today, true, true).await?;
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[aliases(завтра)]
#[description("display tomorrow events from w3info")]
pub async fn tomorrow(ctx: &Context, msg: &Message) -> CommandResult {
  let tomorrow: DateTime<Utc> = Utc::now() + Duration::days(1); 
  tour(ctx, msg, tomorrow).await?;
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  Ok(())
}

#[command]
#[aliases(выходные)]
#[description("display weekends events from w3info")]
pub async fn weekends(ctx: &Context, msg: &Message) -> CommandResult {
  let mut today: DateTime<Utc> = Utc::now();
  if today.weekday() == Weekday::Sun {
    tour_internal(ctx, msg.channel_id, today, true, false).await?;
  } else {
    let is_saturday = today.weekday() == Weekday::Sat;
    if !is_saturday {
      while today.weekday() != Weekday::Sat {
        today += Duration::days(1); 
      }
    }
    tour_internal(ctx, msg.channel_id, today, is_saturday, true).await?;
    let tomorrow: DateTime<Utc> = today + Duration::days(1); 
    tour(ctx, msg, tomorrow).await?;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  Ok(())
}

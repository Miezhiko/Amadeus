use crate::{
  common::{
    msg::{ channel_message }
  }
};

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult,
    macros::command
  },
};

use ical;
use reqwest;

use std::io::BufReader;

use chrono::prelude::*;
use chrono::{ Duration, Utc };

pub fn tour_internal(ctx: &mut Context, msg: &Message, on : DateTime<Utc>, passed_check : bool) -> CommandResult {
  let res = reqwest::blocking::get("https://warcraft3.info/ical-events")?;
  let buf = BufReader::new(res);

  let reader = ical::IcalParser::new(buf);

  let str_date_now = on.format("%Y%m%d").to_string();
  let str_time_now = on.format("%H%M").to_string();

  let mut eventos : Vec<String> = Vec::new();

  for line in reader {
    match line {
      Ok(l) => {
        for e in l.events {
          let mut is_today = false;
          let mut evstr : String = String::new();
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
                      let str_hour = &val[9..11];
                      let str_min = &val[11..13];
                      let msk =
                        if let Ok(str_int) = str_hour.parse::<i32>() {
                          let mut msk_h = str_int + 3;
                          if msk_h >= 24 {
                            msk_h = msk_h - 24;
                          }
                          format!(" ({}:{} MSK)", msk_h.to_string(), str_min)
                        } else { String::from("") };
                      evstr = format!("• {}:{} CET {}", str_hour, str_min, msk);
                    }
                  }
                }
              }
            } else {
              if is_today {
                if ep.name == "SUMMARY" {
                  if let Some(val) = &ep.value {
                    evstr = format!("{} | {}", evstr, val);
                  }
                }
                if ep.name == "DESCRIPTION" {
                  if let Some(val2) = &ep.value {
                    evstr = format!("{}\n<{}>", evstr, val2);
                  }
                }
              }
            }
          }
          if is_today && !evstr.is_empty() {
            eventos.push(evstr);
          }
        }
      },
      Err(e) => error!("Failed to parse calendar line {:?}", e)
    }
  }

  if eventos.len() > 0 {
    let out = eventos.join("\n");
    channel_message(&ctx, &msg, out.as_str());
  }
  Ok(())
}

pub fn tour(ctx: &mut Context, msg: &Message, on : DateTime<Utc>) -> CommandResult {
  tour_internal(ctx, msg, on, false)
}

#[command]
pub fn yesterday(ctx: &mut Context, msg: &Message) -> CommandResult {
  let yesterday : DateTime<Utc> = Utc::now() - Duration::days(1); 
  tour(ctx, msg, yesterday)
}

#[command]
pub fn today(ctx: &mut Context, msg: &Message) -> CommandResult {
  let today : DateTime<Utc> = Utc::now(); 
  tour_internal(ctx, msg, today, true)
}

#[command]
pub fn tomorrow(ctx: &mut Context, msg: &Message) -> CommandResult {
  let tomorrow : DateTime<Utc> = Utc::now() + Duration::days(1); 
  tour(ctx, msg, tomorrow)
}

#[command]
pub fn weekends(ctx: &mut Context, msg: &Message) -> CommandResult {
  let mut today : DateTime<Utc> = Utc::now();
  if today.weekday() == Weekday::Sun {
    channel_message(&ctx, &msg, "Sunday:");
    tour_internal(ctx, msg, today, true)?;
  } else {
    let is_saturday = today.weekday() == Weekday::Sat;
    while today.weekday() != Weekday::Sat {
      today = today + Duration::days(1); 
    }
    channel_message(&ctx, &msg, "Saturday:");
    tour_internal(ctx, msg, today, is_saturday)?;
    let tomorrow : DateTime<Utc> = Utc::now() + Duration::days(1); 
    channel_message(&ctx, &msg, "Sunday:");
    tour(ctx, msg, tomorrow)?;
  }
  Ok(())
}

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

pub fn tour(ctx: &mut Context, msg: &Message, on : DateTime<Utc>) -> CommandResult {
  let res = reqwest::blocking::get("https://warcraft3.info/ical-events")?;
  let buf = BufReader::new(res);

  let reader = ical::IcalParser::new(buf);

  let str_date_now = on.format("%Y%m%d").to_string();

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
                  if str_date_now == str_date {
                    is_today = true;
                    if val.len() >= 14 {
                      let str_hour = &val[9..11];
                      let str_min = &val[11..13];
                      evstr = format!("• {}:{} CET", str_hour, str_min);
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

#[command]
pub fn yesterday(ctx: &mut Context, msg: &Message) -> CommandResult {
  let yesterday : DateTime<Utc> = Utc::now() - Duration::days(1); 
  tour(ctx, msg, yesterday)
}

#[command]
pub fn today(ctx: &mut Context, msg: &Message) -> CommandResult {
  let today : DateTime<Utc> = Utc::now(); 
  tour(ctx, msg, today)
}

#[command]
pub fn tomorrow(ctx: &mut Context, msg: &Message) -> CommandResult {
  let tomorrow : DateTime<Utc> = Utc::now() + Duration::days(1); 
  tour(ctx, msg, tomorrow)
}

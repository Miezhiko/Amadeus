use crate::{
  types::serenity::ChannelLanguage,
  common::msg::{ reply, channel_message },
  collections::base::CASELIST,
  collections::channels::AI_ALLOWED,
  steins::ai::{ cache::KATHOEY
              , bert
              , chain::generate }
};

use serenity::{
  prelude::*,
  model::channel::Message
};

use rand::Rng;

use chat_utils::help::lang;

use tokio::time::{ sleep, Duration };
use async_recursion::async_recursion;

#[async_recursion]
async fn generate_response( ctx: &Context
                          , msg: &Message
                          , gtry: u32
                          , is_response: bool
                          , guild_id: u64 ) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id);
  let message_id = if is_response { Some(msg.id.get()) } else { None };
  if gtry > 0 {
    warn!("Response: failed to generate normal response, try: {gtry}");
  }
  let russian =
    if let Some(ch_lang) = AI_ALLOWED.iter().find(|c| c.id == msg.channel_id.get()) {
      match ch_lang.lang {
        ChannelLanguage::English => {
          false
        },
        ChannelLanguage::Russian => {
          true
        },
        ChannelLanguage::Bilingual => {
          lang::is_russian(&msg.content)
        }
      }
    } else {
      lang::is_russian(&msg.content)
    };
  let rndx: u32 = rand::thread_rng().gen_range(0..30);
  let mut bert_generated = false;
  let in_case = CASELIST.iter().any(|u| *u == msg.author.id.get()
                                     || *u == guild_id);
  let mut answer_option =
    if rndx != 1 && !in_case && gtry < 10 {
      match bert::chat( message_id
                      , msg.channel_id.get()
                      , msg.content.clone()
                      , msg.author.id.get()
                      , guild_id ).await {
        Ok(answer) => {
          bert_generated = true;
          answer },
        Err(why) => {
          error!("Failed to bert chat {why}, input: {}", &msg.content);
          Some( generate(ctx, msg, Some(russian)).await )
        }
      }
    } else {
      if gtry > 9 {
        warn!("Failed to generate normal response after 10 tryes!, msg was: {}", &msg.content);
      }
      Some( generate(ctx, msg, Some(russian)).await )
    };
  if let Some(ref mut answer) = answer_option {
    if russian && !answer.is_empty() && !bert_generated {
      let rndxx: u32 = rand::thread_rng().gen_range(0..2);
      if rndxx == 1 {
        let kathoey = KATHOEY.lock().await;
        let rndxxx: u32 = rand::thread_rng().gen_range(0..30);
        *answer =
          if rndxxx == 1 {
            kathoey.extreme_feminize(answer)
          } else {
            kathoey.feminize(answer)
          };
      }
    }
    *answer = answer.as_str().trim().to_string();
  }
  start_typing.stop();
  #[allow(clippy::manual_filter)]
  if let Some(answer) = answer_option {
    if answer.is_empty() || answer.len() < 3 {
      sleep(Duration::from_millis(100)).await;
      generate_response( ctx
                       , msg
                       , gtry + 1
                       , is_response
                       , guild_id ).await
    } else {
      Some(answer)
    }
  } else {
    None
  }
}

pub async fn chat(ctx: &Context, msg: &Message, guild_id: u64) {
  if let Some(answer) = generate_response(ctx, msg, 0, false, guild_id).await {
    if !answer.is_empty() {
      let rnd = rand::thread_rng().gen_range(0..3);
      if rnd == 1 {
        reply(ctx, msg, &answer).await;
      } else {
        channel_message(ctx, msg, &answer).await;
      }
    }
  }
}

pub async fn response(ctx: &Context, msg: &Message, guild_id: u64) {
  if let Some(answer) = generate_response(ctx, msg, 0, true, guild_id).await {
    if !answer.is_empty() {
      reply(ctx, msg, &answer).await;
    }
  }
}

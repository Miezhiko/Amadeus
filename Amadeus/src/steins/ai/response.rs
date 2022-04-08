use crate::{
  types::serenity::{ IContext
                   , ChannelLanguage },
  common::{ help::lang
          , msg::{ reply, channel_message }
  },
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

use tokio::time::{ sleep, Duration };

use async_recursion::async_recursion;

#[cfg(feature = "torch")]
#[async_recursion]
async fn generate_response( ctx: &Context
                          , msg: &Message
                          , gtry: u32
                          , lsm: bool
                          , is_response: bool ) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let message_id = if is_response { Some(msg.id.0) } else { None };
  if gtry > 0 {
    warn!("Failed to generate normal respons, try: {gtry}");
  }
  let russian =
    if let Some(ch_lang) = AI_ALLOWED.iter().find(|c| c.id == msg.channel_id.0) {
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
  let in_case = CASELIST.iter().any(|u| *u == msg.author.id.0);
  let mut answer_option =
    if rndx != 1 && !in_case && gtry < 10 {
      let text = if russian {
        match bert::ru2en(msg.content.clone(), lsm).await {
          Ok(translated) => translated,
          Err(why) => {
            error!("Failed to translate msg content {why}");
            msg.content.clone()
          }
        }
      } else { msg.content.clone() };
      if msg.content.ends_with('?') {
        let rndxqa: u32 = rand::thread_rng().gen_range(0..2);
        if rndxqa == 1 {
          match bert::ask(text, lsm).await {
            Ok(answer) => {
              bert_generated = true;
              answer },
            Err(why) => {
              error!("Failed to bert ask {why}");
              Some( generate(ctx, msg, Some(russian)).await )
            }
          }
        } else {
          match bert::chat(message_id, msg.channel_id.0, text, msg.author.id.0, lsm).await {
            Ok(answer) => {
              bert_generated = true;
              answer },
            Err(why) => {
              error!("Failed to bert chat with question {why}, input: {}", &msg.content);
              Some( generate(ctx, msg, Some(russian)).await )
            }
          }
        }
      } else {
        match bert::chat(message_id, msg.channel_id.0, text, msg.author.id.0, lsm).await {
          Ok(answer) => {
            bert_generated = true;
            answer },
          Err(why) => {
            error!("Failed to bert chat {why}, input: {}", &msg.content);
            Some( generate(ctx, msg, Some(russian)).await )
          }
        }
      }
    } else {
      if gtry > 9 {
        warn!("Failed to generate normal response after 10 tryes!, msg was: {}", &msg.content);
      }
      Some( generate(ctx, msg, Some(russian)).await )
    };
  if let Some(ref mut answer) = answer_option {
    if russian && !answer.is_empty() {
      if bert_generated {
        match bert::en2ru(answer.clone(), lsm).await {
          Ok(translated) => {
            let rnda: u32 = rand::thread_rng().gen_range(0..10);
            if rnda != 1 {
              let kathoey = KATHOEY.lock().await;
              let rndy: u32 = rand::thread_rng().gen_range(0..30);
              *answer =
                if rndy == 1 {
                  kathoey.extreme_feminize(&translated)
                } else {
                  kathoey.feminize(&translated)
                };
            } else {
              *answer = translated;
            }
          }, Err(why) => {
            error!("Failed to translate answer to Russian {why}");
          }
        }
      } else {
        let rndxx: u32 = rand::thread_rng().gen_range(0..2);
        if rndxx == 1 {
          let kathoey = KATHOEY.lock().await;
          let rndxxx: u32 = rand::thread_rng().gen_range(0..30);
          *answer =
            if rndxxx == 1 {
              kathoey.extreme_feminize(&answer)
            } else {
              kathoey.feminize(&answer)
            };
        }
      }
    }
    *answer = answer.as_str().trim().to_string();
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  if let Some(answer) = answer_option {
    if answer.is_empty() || answer.len() < 3 {
      sleep(Duration::from_millis(100)).await;
      generate_response(ctx, msg, gtry + 1, lsm, is_response).await
    } else {
      Some(answer)
    }
  } else {
    None
  }
}

#[cfg(not(feature = "torch"))]
#[async_recursion]
async fn generate_response( ctx: &Context
                          , msg: &Message
                          , gtry: u32
                          , lsm: bool
                          , is_response: bool ) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  if gtry > 0 {
    warn!("Failed to generate normal respons, try: {gtry}");
  }
  let russian =
    if let Some(ch_lang) = AI_ALLOWED.iter().find(|c| c.id == msg.channel_id.0) {
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
  let mut answer = generate(ctx, msg, Some(russian)).await;
  if russian && !answer.is_empty() {
    let rndxx: u32 = rand::thread_rng().gen_range(0..2);
    if rndxx == 1 {
      let kathoey = KATHOEY.lock().await;
      let rndxxx: u32 = rand::thread_rng().gen_range(0..30);
      answer =
        if rndxxx == 1 {
          kathoey.extreme_feminize(&answer)
        } else {
          kathoey.feminize(&answer)
        };
    }
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  let trimmd = answer.as_str().trim();
  if trimmd.is_empty() || trimmd.len() < 3 {
    sleep(Duration::from_millis(100)).await;
    generate_response(ctx, msg, gtry + 1, lsm, is_response).await
  } else {
    Some(answer)
  }
}

pub async fn chat(ctx: &Context, msg: &Message) {
  let lsm = {
    let data = ctx.data.read().await;
    if let Some(icontext) = data.get::<IContext>() {
      icontext.lazy_static_models
    } else { false }
  };
  if let Some(answer) = generate_response(ctx, msg, 0, lsm, false).await {
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

pub async fn response(ctx: &Context, msg: &Message) {
  let lsm = {
    let data = ctx.data.read().await;
    if let Some(icontext) = data.get::<IContext>() {
      icontext.lazy_static_models
    } else { false }
  };
  if let Some(answer) = generate_response(ctx, msg, 0, lsm, true).await {
    if !answer.is_empty() {
      reply(ctx, msg, &answer).await;
    }
  }
}

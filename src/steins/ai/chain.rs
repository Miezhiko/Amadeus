use crate::{
  types::common::{ ChannelLanguage
                 , AllGuilds },
  common::{ help::lang
          , db::trees::{ register, check_registration }
          , msg::{ reply, channel_message }
  },
  collections::base::{ OBFUSCATION
                     , OBFUSCATION_RU
                     , CASELIST },
  collections::channels::{ AI_LEARN, AI_ALLOWED },
  steins::ai::{ cache::{ CACHE_RU
                       , CACHE_ENG
                       , KATHOEY
                       , RE1, RE2, RE3
                       , NLPR_RULES, NLPR_TOKENIZER
                       , process_message_string
                       , self }
              , boris, uwu, bert }
};

use serenity::{
  prelude::*,
  model::{ channel::{ Message
                    , GuildChannel }
         , id::{ UserId
               , ChannelId
               , GuildId }
         }
};

use markov::Chain;

use rand::Rng;

use tokio::sync::MutexGuard ;

use async_recursion::async_recursion;

use std::collections::HashMap;

pub async fn make_quote(ctx: &Context, msg: &Message, author_id: UserId) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let mut have_something = false;

  let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
  let data = ctx.data.read().await;
  if let Some(servers) = data.get::<AllGuilds>() {
    let server_ids = servers.iter()
                            .map(|srv| GuildId(srv.id))
                            .collect::<Vec<GuildId>>();
    for server in server_ids {
      if let Ok(serv_channels) = server.channels(ctx).await {
        all_channels.extend(serv_channels);
      }
    }
  }

  if !all_channels.is_empty() {
    let mut chain = Chain::new();
    for (chan, _) in all_channels {
      if AI_LEARN.iter().any(|c| c.id == chan.0) {
        if let Ok(messages) = chan.messages(&ctx, |r|
          r.limit(100) // 100 is max
        ).await {
          for mmm in messages {
            if mmm.author.id == author_id && !mmm.content.starts_with('~') {
              let mut result_string = RE1.replace_all(&mmm.content, "").to_string();
              result_string = RE2.replace_all(&result_string, "").to_string();
              result_string = RE3.replace_all(&result_string, "").to_string();
              result_string = result_string.replace(": ", "");
              let is_http = result_string.starts_with("http") && !result_string.starts_with("https://images");
              let result = result_string.trim();
              if !result.is_empty() && !result.contains('$') && !is_http {
                chain.feed_str(result);
                if !have_something {
                  have_something = true;
                }
              }
            }
          }
        }
      }
    }
    if have_something {
      if let Ok(typing) = start_typing {
        typing.stop();
      }
      return Some(chain.generate_str());
    }
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  None
}

pub async fn generate_with_language(ctx: &Context, russian: bool) -> String {
  cache::actualize_cache(ctx, false).await;
  let chain: MutexGuard<Chain<String>> =
    if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  chain.generate_str()
}

pub async fn correct(msg: &str) -> String {
  let nlp = NLPR_RULES.lock().await;
  let tokenizer = NLPR_TOKENIZER.lock().await;
  nlp.correct(&msg, &tokenizer)
}

pub async fn generate(ctx: &Context, msg: &Message, mbrussian: Option<bool>) -> String {
  let msg_content = &msg.content;
  let russian = if let Some(rus) = mbrussian
    { rus } else { lang::is_russian(msg_content) };
  cache::actualize_cache(ctx, false).await;
  let mut chain: MutexGuard<Chain<String>> =
    if russian {
        CACHE_RU.lock().await
      } else {
        CACHE_ENG.lock().await
      };
  if !check_registration(msg.channel_id.0, msg.id.0).await {
    let ch_lang = if russian {
        ChannelLanguage::Russian
      } else {
        ChannelLanguage::English
      };
    if let Some((result, _)) = process_message_string(msg_content, ch_lang) {
      chain.feed_str(&result);
    }
    register(msg.channel_id.0, msg.id.0).await;
  }
  let mut out = chain.generate_str();
  let rndx = rand::thread_rng().gen_range(0..66);
  if rndx == 1 {
    if russian {
      out = boris::spell(&out);
    } else {
      out = uwu::spell(&out);
    }
  }
  if !russian && rndx < 30 {
    out = correct(&out).await;
  }
  out
}

pub fn obfuscate(msg_content: &str) -> String {
  let mut chain = Chain::new();
  let russian = lang::is_russian(msg_content);
  if !russian {
    for confuse in OBFUSCATION.iter() {
      chain.feed_str( confuse );
    }
  } else {
    for confuse in OBFUSCATION_RU.iter() {
      chain.feed_str( confuse );
    }
  }
  chain.feed_str(msg_content);
  let rndx = rand::thread_rng().gen_range(0..6);
  let cahin_string = chain.generate_str();
  if rndx == 1 {
    if russian {
      boris::spell(&cahin_string)
    } else {
      uwu::spell(&cahin_string)
    }
  } else {
    cahin_string
  }
}

#[async_recursion]
async fn generate_response(ctx: &Context, msg: &Message, gtry: u32) -> String {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  if gtry > 0 {
    warn!("Failed to generate normal respons, try: {}", gtry);
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
  let mut answer =
    if rndx != 1 && !in_case && gtry < 10 {
      let text = if russian {
        match bert::ru2en(msg.content.clone()).await {
          Ok(translated) => translated,
          Err(why) => {
            error!("Failed to translate msg content {:?}" , why);
            msg.content.clone()
          }
        }
      } else { msg.content.clone() };
      if msg.content.ends_with('?') {
        let rndxqa: u32 = rand::thread_rng().gen_range(0..2);
        if rndxqa == 1 {
          match bert::ask(text).await {
            Ok(answer) => {
              bert_generated = true;
              answer },
            Err(why) => {
              error!("Failed to bert ask {:?}" , why);
              generate(&ctx, &msg, Some(russian)).await
            }
          }
        } else {
          match bert::chat(text, msg.author.id.0).await {
            Ok(answer) => {
              bert_generated = true;
              answer },
            Err(why) => {
              error!("Failed to bert chat with question {:?}" , why);
              generate(&ctx, &msg, Some(russian)).await
            }
          }
        }
      } else {
        match bert::chat(text, msg.author.id.0).await {
          Ok(answer) => {
            bert_generated = true;
            answer },
          Err(why) => {
            error!("Failed to bert chat {:?}" , why);
            generate(&ctx, &msg, Some(russian)).await
          }
        }
      }
    } else {
      if gtry > 9 {
        warn!("Failed to generate normal response after 10 tryes!, msg was: {}", &msg.content);
      }
      generate(&ctx, &msg, Some(russian)).await
    };
  if russian && !answer.is_empty() {
    if bert_generated {
      match bert::en2ru(answer.clone()).await {
        Ok(translated) => {
          let rnda: u32 = rand::thread_rng().gen_range(0..10);
          if rnda != 1 {
            let kathoey = KATHOEY.lock().await;
            let rndy: u32 = rand::thread_rng().gen_range(0..30);
            answer =
              if rndy == 1 {
                kathoey.extreme_feminize(&translated)
              } else {
                kathoey.feminize(&translated)
              };
          } else {
            answer = translated;
          }
        }, Err(why) => {
          error!("Failed to translate answer to Russian {:?}" , why);
        }
      }
    } else {
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
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  let trimmd = answer.as_str().trim();
  if trimmd.is_empty() || trimmd.len() < 3 {
    generate_response(ctx, msg, gtry + 1).await
  } else {
    answer
  }
}

pub async fn chat(ctx: &Context, msg: &Message) {
  let answer = generate_response(ctx, msg, 0).await;
  if !answer.is_empty() {
    let rnd = rand::thread_rng().gen_range(0..3);
    if rnd == 1 {
      reply(&ctx, &msg, &answer).await;
    } else {
      channel_message(&ctx, &msg, &answer).await;
    }
  }
}

pub async fn response(ctx: &Context, msg: &Message) {
  let answer = generate_response(ctx, msg, 0).await;
  if !answer.is_empty() {
    reply(&ctx, &msg, &answer).await;
  }
}

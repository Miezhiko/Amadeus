use crate::{
  types::common::ChannelLanguage,
  common::{ help::lang
          , msg::{ reply, channel_message }
  },
  collections::base::{ OBFUSCATION
                     , OBFUSCATION_RU },
  collections::channels::{ AI_LEARN, AI_ALLOWED },
  steins::ai::{ cache::{ CACHE_RU
                       , CACHE_ENG
                       , KATHOEY
                       , RE1, RE2, RE3
                       , self }
              , boris, uwu, bert }
};

use serenity::{
  prelude::*,
  model::{ channel::Message
         , id::UserId }
};

use markov::Chain;

use rand::Rng;

use tokio::sync::MutexGuard ;

use async_recursion::async_recursion;

pub async fn make_quote(ctx: &Context, msg: &Message, author_id: UserId) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let mut have_something = false;
  if let Some(guild_id) = msg.guild_id {
    let mut chain = Chain::new();
    if let Ok(channels) = guild_id.channels(&ctx).await {
      for (chan, _) in channels {
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
  cache::actualize_cache(ctx).await;
  let chain : MutexGuard<Chain<String>> =
    if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  chain.generate_str()
}

pub async fn generate(ctx: &Context, msg: &Message, mbrussian: Option<bool>) -> String {
  let msg_content = &msg.content;
  let russian = if let Some(rus) = mbrussian
    { rus } else { lang::is_russian(msg_content) };
  cache::actualize_cache(ctx).await;
  let chain : MutexGuard<Chain<String>> =
  if russian {
      CACHE_RU.lock().await
    } else {
      CACHE_ENG.lock().await
    };
  let mut out = chain.generate_str();
  let rndx = rand::thread_rng().gen_range(0..66);
  if rndx == 1 {
    if russian {
      out = boris::spell(&out);
    } else {
      out = uwu::spell(&out);
    }
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
async fn generate_response(ctx: &Context, msg: &Message) -> String {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
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
  let rndx: u32 = rand::thread_rng().gen_range(0..20);
  let mut bert_generated = false;
  let mut answer =
    if rndx != 1 {
      let text = if russian {
        if let Ok(translated) = bert::ru2en(msg.content.clone()).await {
          translated
        } else { msg.content.clone() }
        } else { msg.content.clone() };
      if msg.content.ends_with('?') {
        let rndxqa: u32 = rand::thread_rng().gen_range(0..2);
        if rndxqa == 1 {
          if let Ok(answer) = bert::ask(text).await {
            bert_generated = true;
            answer
          } else {
            generate(&ctx, &msg, Some(russian)).await
          }
        } else if let Ok(answer) = bert::chat(text, msg.author.id.0).await {
          bert_generated = true;
          answer
        } else {
          generate(&ctx, &msg, Some(russian)).await
        }
      } else if let Ok(answer) = bert::chat(text, msg.author.id.0).await {
        bert_generated = true;
        answer
      } else {
        generate(&ctx, &msg, Some(russian)).await
      }
    } else {
      generate(&ctx, &msg, Some(russian)).await
    };
  if russian {
    if bert_generated {
      if let Ok(translated) = bert::en2ru(answer.clone()).await {
        // feminize translated text
        let kathoey = KATHOEY.lock().await;
        let rndy : u32 = rand::thread_rng().gen_range(0..15);
        answer =
          if rndy == 1 {
            kathoey.extreme_feminize(&translated)
          } else {
            kathoey.feminize(&translated)
          };
      }
    } else {
      let rndxx : u32 = rand::thread_rng().gen_range(0..2);
      if rndxx == 1 {
        let kathoey = KATHOEY.lock().await;
        let rndxxx : u32 = rand::thread_rng().gen_range(0..15);
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
    generate_response(ctx, msg).await
  } else {
    answer
  }
}

pub async fn chat(ctx: &Context, msg: &Message) {
  let answer = generate_response(ctx, msg).await;
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
  let answer = generate_response(ctx, msg).await;
  if !answer.is_empty() {
    reply(&ctx, &msg, &answer).await;
  }
}

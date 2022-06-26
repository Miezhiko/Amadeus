use crate::{
  types::serenity::{ ChannelLanguage
                   , AllGuilds },
  common::{ help::lang
          , constants::PREFIX
          , db::trees::messages::{ register, check_registration }
  },
  collections::base::{ OBFUSCATION
                     , OBFUSCATION_RU },
  collections::channels::AI_LEARN,
  steins::ai::{ cache::{ CACHE_RU
                       , CACHE_ENG
                       , process_message_string
                       , self }
              , boris, uwu }
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

use tokio::sync::MutexGuard;

use std::collections::HashMap;

use mozart::bert::{ RE1, RE2, RE3 };

pub async fn make_quote(ctx: &Context, msg: &Message, author_id: UserId) -> Option<String> {
  let start_typing = ctx.http.start_typing(msg.channel_id.0.get());
  let mut have_something = false;

  let mut all_channels: HashMap<ChannelId, GuildChannel> = HashMap::new();
  let data = ctx.data.read().await;
  if let Some(servers) = data.get::<AllGuilds>() {
    let server_ids = servers.iter()
                            .map(|srv| GuildId( to_nzu!( srv.id ) ))
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
      if AI_LEARN.iter().any(|c| c.id == chan.0.get()) {
        if let Ok(messages) = chan.messages(&ctx, |r|
          r.limit(100) // 100 is max
        ).await {
          for mmm in messages {
            if mmm.author.id == author_id && !mmm.content.starts_with(PREFIX) {
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
  if !check_registration(msg.channel_id.0.get(), msg.id.0.get()).await {
    let ch_lang = if russian {
        ChannelLanguage::Russian
      } else {
        ChannelLanguage::English
      };
    if let Some((result, _)) = process_message_string(msg_content, ch_lang) {
      chain.feed_str(&result);
    }
    register(msg.channel_id.0.get(), msg.id.0.get()).await;
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

use crate::salieri::SALIERI;

use anyhow::Result;
use rand::Rng;

use strauss::{
  bert::{ process_message_for_gpt
        , GPT_LIMIT
        , chat::CHAT_GPT2
        , qa::ASK
        , code::CODEBERT },
  chat::CHAT
};

async fn salieri_request<T>( sig: celery::task::Signature<T>
                           ) -> Result<Option<String>>
                           where T: celery::task::Task {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(sig).await?;
    Ok(None)
  } else {
    Err(anyhow!("BERT: Failed to connecto to Salieri"))
  }
}

async fn chat_gpt2( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> Result<Option<String>> {
  salieri_request(CHAT_GPT2::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn ask( msg: Option<u64>
                , chan: u64
                , something: String
                , user_id: u64
                , lsm: bool
                , russian: bool ) -> Result<Option<String>> {
  salieri_request(ASK::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn codebert( msg: Option<u64>
                     , chan: u64
                     , something: String
                     , user_id: u64
                     , lsm: bool
                     , russian: bool ) -> Result<Option<String>> {
  salieri_request(CODEBERT::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn chatrs( msg: Option<u64>
                   , chan: u64
                   , something: String
                   , user_id: u64
                   , lsm: bool
                   , russian: bool ) -> Result<Option<String>> {
  salieri_request(CHAT::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn chat( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool
                 , guild_id: u64 ) -> Result<Option<String>> {
  let wlmt = if guild_id == 611822838831251466
                { 32 }
           else { 5 };
  let rndx = if user_id == 510368731378089984
                { 3 }
           else if wlmt > 0
                 { rand::thread_rng().gen_range(0..wlmt) }
            else { 0 };
  let mut input =
    if rndx < 3 { process_message_for_gpt(&something) }
           else { something };
  if rndx < 3 {
    if russian {
      match strauss::bert::translation::ru2en(input.clone()).await {
        Ok(translated) => input = translated,
        Err(why) => {
          error!("Failed to translate msg content {why}");
        }
      };
    }
    if input.len() > GPT_LIMIT {
      if let Some((i, _)) = input.char_indices().rev().nth(GPT_LIMIT) {
        input = input[i..].to_string();
      }
    }
    if input.is_empty() {
      return Err(anyhow!("empty input"));
    }
  }
  match rndx {
    0 => codebert   (msg, chan, input, user_id, lsm, russian).await,
    1 => chat_gpt2  (msg, chan, input, user_id, lsm, russian).await,
    2 => ask        (msg, chan, input, user_id, lsm, russian).await,
    _ => chatrs     (msg, chan, input, user_id, lsm, russian).await
  }
}

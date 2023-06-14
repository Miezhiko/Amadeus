use crate::salieri::SALIERI;

use anyhow::Result;
use rand::Rng;

use strauss::{
  bert::{ process_message_for_gpt
        , GPT_LIMIT
        , chat::CHAT_GPT2
        , qa::ASK
        , neo::CHAT_NEO
        , summarization::SUMMARIZE
        , xlnet::XLNET
        , code::CODEBERT
        , gptj::GPTJ },
  wagner::WAGNER
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

async fn chat_neo( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> Result<Option<String>> {
  salieri_request(CHAT_NEO::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn ask( msg: Option<u64>
                , chan: u64
                , something: String
                , user_id: u64
                , lsm: bool
                , russian: bool ) -> Result<Option<String>> {
  salieri_request(ASK::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn summarize( msg: Option<u64>
                      , chan: u64
                      , something: String
                      , user_id: u64
                      , lsm: bool
                      , russian: bool ) -> Result<Option<String>> {
  salieri_request(SUMMARIZE::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn xlnet( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> Result<Option<String>> {
  salieri_request(XLNET::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn codebert( msg: Option<u64>
                     , chan: u64
                     , something: String
                     , user_id: u64
                     , lsm: bool
                     , russian: bool ) -> Result<Option<String>> {
  salieri_request(CODEBERT::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn gptj( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> Result<Option<String>> {
  salieri_request(GPTJ::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn wagner( msg: Option<u64>
                   , chan: u64
                   , something: String
                   , user_id: u64
                   , lsm: bool
                   , russian: bool ) -> Result<Option<String>> {
  salieri_request(WAGNER::new(msg, chan, something, user_id, lsm, russian)).await
}

pub async fn chat( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> Result<Option<String>> {
  let rndx = rand::thread_rng().gen_range(0..32);
  let mut input = process_message_for_gpt(&something);
  if input.len() > GPT_LIMIT {
    if let Some((i, _)) = input.char_indices().rev().nth(GPT_LIMIT) {
      input = input[i..].to_string();
    }
  }
  if input.is_empty() {
    return Err(anyhow!("empty input"));
  }
  match rndx {
    0 => chat_neo   (msg, chan, input, user_id, lsm, russian).await,
    1 => summarize  (msg, chan, input, user_id, lsm, russian).await,
    2 => xlnet      (msg, chan, input, user_id, lsm, russian).await,
    3 => codebert   (msg, chan, input, user_id, lsm, russian).await,
    4 => gptj       (msg, chan, input, user_id, lsm, russian).await,
    5 => chat_gpt2  (msg, chan, input, user_id, lsm, russian).await,
    _ => wagner     (msg, chan, input, user_id, lsm, russian).await
  }
}

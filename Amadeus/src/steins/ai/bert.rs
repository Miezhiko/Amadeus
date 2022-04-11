use crate::common::salieri::SALIERI;

use anyhow::Result;
use rand::Rng;

use mozart::{
  bert::{ process_message_for_gpt
        , GPT_LIMIT
        , chat::CHAT_GPT2
        , qa::ASK
        , neo::CHAT_NEO }
};

async fn chat_gpt2( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool ) -> Result<Option<String>> {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(
      CHAT_GPT2::new(msg, chan, something, user_id, lsm)
    ).await?;
    Ok(None)
  } else {
    Err(anyhow!("chat_gpt2: failed to connecto to Salieri"))
  }
}

async fn chat_neo( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , lsm: bool ) -> Result<Option<String>> {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(
      CHAT_NEO::new(msg, chan, something, lsm)
    ).await?;
    Ok(None)
  } else {
    Err(anyhow!("chat_neo: failed to connecto to Salieri"))
  }
}

pub async fn ask( msg: Option<u64>
                , chan: u64
                , something: String
                , lsm: bool ) -> Result<Option<String>> {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(
      ASK::new(msg, chan, something, lsm)
    ).await?;
    Ok(None)
  } else {
    Err(anyhow!("chat_neo: failed to connecto to Salieri"))
  }
}

pub async fn chat( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool) -> Result<Option<String>> {
  let rndx = rand::thread_rng().gen_range(0..7);
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
    0 => chat_neo(msg, chan, input, lsm).await,
    _ => chat_gpt2(msg, chan, input, user_id, lsm).await
  }
}

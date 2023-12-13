use crate::{
  types::ChatResponse,
  cache::*,
  prelude::*,
  bert::LUKASHENKO
};

use celery::prelude::*;

use rust_bert::pipelines::{
  conversation::ConversationManager
};

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use tokio::task;
use chrono::Utc;

use rand::seq::SliceRandom;
use async_recursion::async_recursion;

pub async fn reinit() {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
}

pub async fn chat_gpt2(something: String, user_id: u64, lsm: bool) -> anyhow::Result<String> {
  info!("Generating GPT2 response");
  let cache_eng_hs = CACHE_ENG_STR.lock().await;
  let mut conversation = CONVMODEL.lock().await;
  if conversation.is_none() {
    *conversation = Some( conv_model_loader() );
  }
  if ! lsm {
    let mut conv_model_used = CONVMODEL_USED.lock().await;
    *conv_model_used = Some(Utc::now());
  }
  let mut chat_context = CHAT_CONTEXT.lock().await;
  task::spawn_blocking(move || {
    if let Some(conversation_model) = &mut *conversation {
      let cache_eng_vec = cache_eng_hs.iter().collect::<Vec<&String>>();
      let output =
        if let Some((tracking_conversation, x)) = chat_context.get_mut(&user_id) {
          if *x > 5 {
            chat_context.remove(&user_id);
            let mut conversation_manager = ConversationManager::new();
            let cache_slices = cache_eng_vec
                              .choose_multiple(&mut rand::thread_rng(), 32)
                              .map(AsRef::as_ref).collect::<Vec<&str>>();
            let encoded_history = conversation_model.encode_prompts(&cache_slices);
            let conv_id = conversation_manager.create(&something);
            if let Some(cm) = conversation_manager.get(&conv_id) {
              cm.load_from_history(&cache_slices, &encoded_history);
            }
            chat_context.insert( user_id
                               , ( conversation_manager, 0 )
                               );
            if let Some(chat_cont) = chat_context.get_mut(&user_id) {
              let (registered_conversation, _) = chat_cont;
              conversation_model.generate_responses(registered_conversation)
            } else {
              return Err(anyhow!("Failed to cregister conversation for {}", &user_id));
            }
          } else {
            tracking_conversation.create(&something);
            *x += 1;
            conversation_model.generate_responses(tracking_conversation)
          }
        } else {
          let mut conversation_manager = ConversationManager::new();
          let cache_slices = cache_eng_vec
                            .choose_multiple(&mut rand::thread_rng(), 5)
                            .map(AsRef::as_ref).collect::<Vec<&str>>();
          let encoded_history = conversation_model.encode_prompts(&cache_slices);
          let conv_id = conversation_manager.create(&something);
          if let Some(cm) = conversation_manager.get(&conv_id) {
            cm.load_from_history(&cache_slices, &encoded_history);
          }

          chat_context.insert( user_id, ( conversation_manager, 0 ) );

          if let Some(chat_cont) = chat_context.get_mut(&user_id) {
            let (registered_conversation, _) = chat_cont;
            conversation_model.generate_responses(registered_conversation)
          } else {
            return Err(anyhow!("Failed to cregister conversation for {}", &user_id));
          }
        };

      let out_values = output.values()
                             .cloned()
                             .map(str::to_string)
                             .collect::<Vec<String>>();

      if out_values.is_empty() {
        Err(anyhow!("no output from GPT 2 model"))
      } else {
        let answer = out_values[0].clone();
        if answer.is_empty() || answer.len() == 1 {
          Err(anyhow!("GPT 2: bad answer, I don't like it"))
        } else {
          Ok(answer)
        }
      }
    } else {
      Err(anyhow!("Empty GPT 2 model"))
    }
  }).await.unwrap()
}

#[async_recursion]
pub async fn chat_gpt2_send( msg: Option<u64>
                           , chan: u64
                           , something: String
                           , user_id: u64
                           , lsm: bool
                           , russian: bool
                           , gtry: u32 ) -> anyhow::Result<()> {
  if gtry > 0 {
    warn!("GPT2: trying again: {gtry}");
  }
  match chat_gpt2(something.clone(), user_id, lsm).await {
    Ok(result) => {
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      let package = ChatResponse {
        message: msg,
        channel: chan,
        response: result,
        russian
      };
      let encoded = bincode::encode_to_vec(package, BINCODE_CONFIG)?;
      lukashenko.write_all(&encoded)?;
      Ok(())
    }, Err(why) => {
      error!("GPT2: Failed to generate response: {why}");
      if gtry > 9 {
        error!("GPT2: failed to generate response 10 times!");
        Err( why )
      } else {
        chat_gpt2_send(msg, chan, something, user_id, lsm, russian, gtry + 1).await
      }
    }
  }
}

#[celery::task]
pub async fn CHAT_GPT2( msg: Option<u64>
                      , chan: u64
                      , something: String
                      , user_id: u64
                      , lsm: bool
                      , russian: bool ) -> TaskResult<()> {
  if let Err(why) = chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await {
    error!("Failed to generate response, {why}");
    // ignore this
    // Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("GPT2 response sent to {LUKASHENKO}!");
  }
  Ok(())
}

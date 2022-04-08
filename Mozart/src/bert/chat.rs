use crate::cache::*;

use celery::{ self, prelude::* };

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

pub const LUKASHENKO: &str = "lukashenko";

pub async fn reinit() {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
}

// temporary pub!
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
        Ok(out_values[0].clone())
      }
    } else {
      Err(anyhow!("Empty GPT 2 model"))
    }
  }).await.unwrap()
}

async fn chat_gpt2_send( msg: Option<u64>
                       , chan: u64
                       , something: String
                       , user_id: u64
                       , lsm: bool ) -> anyhow::Result<()> {
  let result = chat_gpt2(something, user_id, lsm).await?;
  let temp_dir = std::env::temp_dir();
  let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
  let config = bincode::config::standard();
  let package = crate::types::ChatResponse {
    message: msg,
    channel: chan,
    response: result
  };
  let encoded = bincode::	encode_to_vec(&package, config)?;
  lukashenko.write_all(&encoded)?;
  Ok(())
}

#[celery::task]
pub async fn CHAT_GPT2( msg: Option<u64>
                      , chan: u64
                      , something: String
                      , user_id: u64
                      , lsm: bool ) -> TaskResult<()> {
  if let Err(why) = chat_gpt2_send(msg, chan, something, user_id, lsm).await {
    error!("Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("GPT2 response sent to {LUKASHENKO}!");
    Ok(())
  }
}

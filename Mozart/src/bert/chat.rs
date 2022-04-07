use celery::{ self, prelude::* };

use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig }
};

use std::collections::{ HashSet, HashMap };

use tch::Device;
use tokio::{ task, sync::Mutex };
use once_cell::sync::Lazy;
use chrono::{ Utc, DateTime };

use rand::seq::SliceRandom;

pub static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);

pub static CACHE_ENG_STR: Lazy<Mutex<HashSet<String>>> =
  Lazy::new(|| Mutex::new(HashSet::new()));

fn conv_model_loader() -> ConversationModel {
  ConversationModel::new(
    ConversationConfig {
      min_length: 3,
      max_length: 64,
      min_length_for_response: 5,
      device: *DEVICE,
      ..Default::default()
    }
  ).unwrap()
}

pub static CONVMODEL: Lazy<Mutex<Option<ConversationModel>>> =
  Lazy::new(|| Mutex::new(Some(conv_model_loader())));

pub static CONVMODEL_USED: Lazy<Mutex<Option<DateTime<Utc>>>> =
  Lazy::new(|| Mutex::new(None));

#[allow(clippy::type_complexity)]
pub static CHAT_CONTEXT: Lazy<Mutex<HashMap<u64, (ConversationManager, u32)>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

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

#[celery::task]
pub async fn CHAT_GPT2(something: String, user_id: u64, lsm: bool) -> TaskResult<String> {
  let result = chat_gpt2(something, user_id, lsm).await;
  match result {
    Ok(r) => Ok(r),
    Err(why) => {
      error!("Failed to generate response, {why}");
      Err( TaskError::ExpectedError( why.to_string() ) )
    }
  }
}

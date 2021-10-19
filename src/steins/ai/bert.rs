use crate::{
  steins::ai::cache::{
    CACHE_ENG_STR,
    process_message_for_gpt
  }
};

use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig },
  question_answering::{ QaInput
                      , QuestionAnsweringModel
                      , QuestionAnsweringConfig },
  translation::{ Language
               , TranslationModelBuilder
               , TranslationModel }
};

use tch::Device;
use tokio::{ task, sync::Mutex };
use once_cell::sync::Lazy;
use chrono::{ Utc, DateTime };

use std::collections::HashMap;

use anyhow::Result;

use rand::{ seq::SliceRandom, Rng };

use super::neo::chat_neo;

pub static TRANSLATION_LIMIT: usize = 512;
static GPT_LIMIT: usize = 1000;

// models
pub static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);

pub fn enru_model_loader() -> TranslationModel {
  TranslationModelBuilder::new()
    .with_source_languages(vec![Language::English, Language::Russian])
    .with_target_languages(vec![Language::English, Language::Russian])
    .with_device(Device::cuda_if_available())
    .create_model().unwrap()
}

pub static ENRUMODEL: Lazy<Mutex<Option<TranslationModel>>> =
  Lazy::new(|| Mutex::new( Some( enru_model_loader() ) ) );

pub static ENRUMODEL_USED: Lazy<Mutex<Option<DateTime<Utc>>>> =
  Lazy::new(|| Mutex::new(None));

fn qa_model_loader() -> QuestionAnsweringModel {
  QuestionAnsweringModel::new(
    QuestionAnsweringConfig {
      lower_case: false,
      device: *DEVICE,
      ..Default::default()
    }
  ).unwrap()
}

static QAMODEL: Lazy<Mutex<Option<QuestionAnsweringModel>>> =
  Lazy::new(|| Mutex::new(Some(qa_model_loader())));

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
pub static CHAT_CONTEXT: Lazy<Mutex<HashMap<u64, (ConversationManager, u32, u32)>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn reinit() {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
}

pub async fn en2ru(text: String, lsm: bool) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let mut enru_model = ENRUMODEL.lock().await;
  if enru_model.is_none() {
    *enru_model = Some( enru_model_loader() );
  }
  if !lsm {
    let mut enru_model_model_used = ENRUMODEL_USED.lock().await;
    *enru_model_model_used = Some(Utc::now());
  }
  task::spawn_blocking(move || {
    if let Some(en2ru_model) = &mut *enru_model {
      let mut something = text;
      if something.len() > TRANSLATION_LIMIT {
        if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
          something = something[i..].to_string();
        }
      }
      let output = en2ru_model.translate(&[something.as_str()], Some(Language::English)
                                                              , Language::Russian)?;
      if output.is_empty() {
        Ok(something)
      } else {
        Ok(output[0].clone())
      }
    } else {
      Err(anyhow!("Empty ENRU model"))
    }
  }).await.unwrap()
}

pub async fn ru2en(text: String, lsm: bool) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let mut enru_model = ENRUMODEL.lock().await;
  if enru_model.is_none() {
    *enru_model = Some( enru_model_loader() );
  }
  if ! lsm {
    let mut enru_model_model_used = ENRUMODEL_USED.lock().await;
    *enru_model_model_used = Some(Utc::now());
  }
  task::spawn_blocking(move || {
    if let Some(ru2en_model) = &mut *enru_model {
      let mut something = text;
      if something.len() > TRANSLATION_LIMIT {
        if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
          something = something[i..].to_string();
        }
      }
      let output = ru2en_model.translate(&[something.as_str()], Some(Language::Russian)
                                                              , Language::English)?;
      if output.is_empty() {
        Ok(something)
      } else {
        let translation = &output[0];
        Ok(translation.clone())
      }
    } else {
      Err(anyhow!("Empty ENRU model"))
    }
  }).await.unwrap()
}

pub async fn ru2en_many(texts: Vec<String>, lsm: bool) -> Result<Vec<String>> {
  if texts.is_empty() {
    return Ok(vec![]);
  }
  let mut enru_model = ENRUMODEL.lock().await;
  if enru_model.is_none() {
    *enru_model = Some( enru_model_loader() );
  }
  if ! lsm {
    let mut enru_model_model_used = ENRUMODEL_USED.lock().await;
    *enru_model_model_used = Some(Utc::now());
  }
  task::spawn_blocking(move || {
    if let Some(ru2en_model) = &mut *enru_model {
      let ttt = texts.iter().map(|t| t.as_str()).collect::<Vec<&str>>();
      let output = ru2en_model.translate(&ttt, Some(Language::Russian)
                                             , Language::English)?;
      if output.is_empty() {
        Ok(Vec::new())
      } else {
        Ok(output)
      }
    } else {
      Err(anyhow!("Empty ENRU model"))
    }
  }).await.unwrap()
}

pub async fn ask(msg_content: String, lsm: bool) -> Result<String> {
  info!("Generating GPT2 QA response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let mut qa = QAMODEL.lock().await;
  if qa.is_none() {
    *qa = Some( qa_model_loader() );
  }
  let mut question = process_message_for_gpt(&msg_content);
  if question.len() > GPT_LIMIT {
    if let Some((i, _)) = question.char_indices().rev().nth(GPT_LIMIT) {
      question = question[i..].to_string();
    }
  }
  let mut cache =
    if cache_eng_vec.is_empty() {
      String::from("HUMBA")
    } else {
      cache_eng_vec.iter().collect::<Vec<&String>>()
        .choose_multiple(&mut rand::thread_rng(), 25)
        .map(AsRef::as_ref).collect::<Vec<&str>>()
        .join(" ")
    };
  if question.len() + cache.len() > GPT_LIMIT {
    if let Some((i, _)) = cache.char_indices().rev().nth(GPT_LIMIT - question.len()) {
      cache = cache[i..].to_string();
    }
  }
  task::spawn_blocking(move || {
    if let Some(qa_model) = &mut *qa {
      let qa_input = QaInput {
        question, context: cache
      };
      // Get answer
      let answers = qa_model.predict(&[qa_input], 1, 32);
      if ! lsm {
        *qa = None;
      }
      if answers.is_empty() {
        Err(anyhow!("no output from GPT QA model"))
      } else {
        let my_answers = &answers[0];

        // we have several answers (hope they sorted by score)
        let answer = &my_answers[0];
        Ok(answer.answer.clone())
      }
    } else {
      Err(anyhow!("Empty QA model"))
    }
  }).await.unwrap()
}

async fn chat_gpt2(something: String, user_id: u64, lsm: bool) -> Result<String> {
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
        if let Some((tracking_conversation, passed, x)) = chat_context.get_mut(&user_id) {
          if *x > 5 {
            chat_context.remove(&user_id);
            let mut conversation_manager = ConversationManager::new();
            let cache_slices = cache_eng_vec
                              .choose_multiple(&mut rand::thread_rng(), 32)
                              .map(AsRef::as_ref).collect::<Vec<&str>>();
            let encoded_history = conversation_model.encode_prompts(&cache_slices);
            let conv_id = conversation_manager.create(&something);
            if let Some(cm) = conversation_manager.get(&conv_id) {
              cm.load_from_history(cache_slices, encoded_history);
            }
            chat_context.insert( user_id
                               , ( conversation_manager, 0, 0 )
                               );
            if let Some(chat_cont) = chat_context.get_mut(&user_id) {
              let (registered_conversation, _, _) = chat_cont;
              conversation_model.generate_responses(registered_conversation)
            } else {
              return Err(anyhow!("Failed to cregister conversation for {}", &user_id));
            }
          } else {
            tracking_conversation.create(&something);
            *passed = 0; *x += 1;
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
            cm.load_from_history(cache_slices, encoded_history);
          }

          chat_context.insert( user_id
                             , ( conversation_manager, 0, 0 )
                             );

          if let Some(chat_cont) = chat_context.get_mut(&user_id) {
            let (registered_conversation, _, _) = chat_cont;
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

pub async fn chat(something: String, user_id: u64, lsm: bool) -> Result<String> {
  let rndx = rand::thread_rng().gen_range(0..4);
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
    0 => chat_neo(input, lsm).await,
    _ => chat_gpt2(input, user_id, lsm).await
  }
}

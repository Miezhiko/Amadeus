use crate::steins::ai::cache::CACHE_ENG_STR;

use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig },
  question_answering::{ QaInput
                      , QuestionAnsweringModel
                      , QuestionAnsweringConfig },
  translation::{ Language
               , TranslationConfig
               , TranslationModel }
};

use tch::Device;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use std::collections::HashMap;

use anyhow::Result;

use rand::{ seq::SliceRandom, Rng };

use super::neo::chat_neo;

// models
static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);
pub static EN2RUMODEL: Lazy<Mutex<TranslationModel>> =
  Lazy::new(||
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::EnglishToRussian, *DEVICE)
    ).unwrap()));
pub static RU2ENMODEL: Lazy<Mutex<TranslationModel>> =
  Lazy::new(||
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::RussianToEnglish, *DEVICE)
    ).unwrap()));
pub static QAMODEL: Lazy<Mutex<QuestionAnsweringModel>> =
  Lazy::new(||
    Mutex::new(QuestionAnsweringModel::new(
      QuestionAnsweringConfig {
        lower_case: false,
        device: *DEVICE,
        ..Default::default()
      }
    ).unwrap()));
pub static CONVMODEL: Lazy<Mutex<ConversationModel>> =
  Lazy::new(||
    Mutex::new(ConversationModel::new(
      ConversationConfig {
        min_length: 2,
        max_length: 100,
        min_length_for_response: 5,
        device: *DEVICE,
        ..Default::default()
      }
    ).unwrap()));

#[allow(clippy::type_complexity)]
pub static CHAT_CONTEXT: Lazy<Mutex<HashMap<u64, (ConversationManager, u32, u32)>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn reinit() {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
}

pub async fn en2ru(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let en2ru_model = EN2RUMODEL.lock().await;
  let output = en2ru_model.translate(&[text.as_str()]);
  if output.is_empty() {
    error!("Failed to translate with TranslationConfig EnglishToRussian");
    Ok(text)
  } else {
    Ok(output[0].clone())
  }
}

pub async fn ru2en(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let ru2en_model = RU2ENMODEL.lock().await;
  let output = ru2en_model.translate(&[text.as_str()]);
  if output.is_empty() {
    error!("Failed to translate with TranslationConfig RussianToEnglish");
    Ok(text)
  } else {
    let translation = &output[0];
    Ok(translation.clone())
  }
}

pub async fn ru2en_many(texts: Vec<String>) -> Result<Vec<String>> {
  if texts.is_empty() {
    return Ok(vec![]);
  }
  let ru2en_model = EN2RUMODEL.lock().await;
  let ttt = texts.iter().map(|t| t.as_str()).collect::<Vec<&str>>();
  let output = ru2en_model.translate(&ttt);
  if output.is_empty() {
    error!("Failed to translate with TranslationConfig RussianToEnglish");
    Ok(Vec::new())
  } else {
    Ok(output)
  }
}

pub async fn ask(question: String) -> Result<String> {
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let cache = 
    if cache_eng_vec.is_empty() {
      String::from("HUMBA")
    } else {
      cache_eng_vec
        .choose_multiple(&mut rand::thread_rng(), 100)
        .map(AsRef::as_ref).collect::<Vec<&str>>()
        .join(" ")
    };
  let qa_model = QAMODEL.lock().await;
  let qa_input = QaInput {
    question, context: cache
  };
  // Get answer
  let answers = qa_model.predict(&[qa_input], 1, 32);
  if answers.is_empty() {
    error!("Failed to ansewer with QuestionAnsweringModel");
    // TODO: error should be here
    Ok(String::new())
  } else {
    let my_answers = &answers[0];

    // we have several answers (hope they sorted by score)
    let answer = &my_answers[0];
    Ok(answer.answer.clone())
  }
}

async fn chat_gpt2(something: String, user_id: u64) -> Result<String> {
  info!("Generating GPT2 response");
  let conversation_model = CONVMODEL.lock().await;
  let mut chat_context = CHAT_CONTEXT.lock().await;
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let output =
    if let Some((tracking_conversation, passed, x)) = chat_context.get_mut(&user_id) {
      if *x > 100 {
        chat_context.remove(&user_id);

        let mut conversation_manager = ConversationManager::new();
        let cache_slices = cache_eng_vec
                        .choose_multiple(&mut rand::thread_rng(), 32)
                        .map(AsRef::as_ref).collect::<Vec<&str>>();
        let encoded_history = conversation_model.encode_prompts(&cache_slices);
        let conv_id = conversation_manager.create(&something);
        conversation_manager.get(&conv_id).unwrap().load_from_history(cache_slices, encoded_history);

        chat_context.insert( user_id
                            , ( conversation_manager, 0, 0 )
                            );

        let (registered_conversation, _, _) =
          chat_context.get_mut(&user_id).unwrap();
        conversation_model.generate_responses(registered_conversation)
      } else {
        tracking_conversation.create(&something);
        *passed = 0; *x += 1;
        conversation_model.generate_responses(tracking_conversation)
      }
    } else {
      let mut conversation_manager = ConversationManager::new();
      let cache_slices = cache_eng_vec
                        .choose_multiple(&mut rand::thread_rng(), 10)
                        .map(AsRef::as_ref).collect::<Vec<&str>>();
      let encoded_history = conversation_model.encode_prompts(&cache_slices);
      let conv_id = conversation_manager.create(&something);
      conversation_manager.get(&conv_id).unwrap().load_from_history(cache_slices, encoded_history);
      chat_context.insert( user_id
                          , ( conversation_manager, 0, 0 )
                          );
      let (registered_conversation, _, _) =
        chat_context.get_mut(&user_id).unwrap();
      conversation_model.generate_responses(registered_conversation)
    };

  let out_values = output.values()
                          .cloned()
                          .map(str::to_string)
                          .collect::<Vec<String>>();

  if out_values.is_empty() {
    error!("Failed to chat with ConversationModel");
    // TODO: error should be here
    Ok(String::new())
  } else {
    // just get first
    let answer = &out_values[0];

    Ok(answer.clone())
  }
}

pub async fn chat(something: String, user_id: u64) -> Result<String> {
  let rndx = rand::thread_rng().gen_range(0..4);
  match rndx {
    0 => chat_neo(something).await,
    _ => chat_gpt2(something, user_id).await
  }
}

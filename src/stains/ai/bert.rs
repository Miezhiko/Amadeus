use crate::stains::ai::chain::CACHE_ENG_STR;

use rust_bert::pipelines::conversation::{ ConversationManager, ConversationModel };
use rust_bert::pipelines::question_answering::{ QaInput, QuestionAnsweringModel };
use rust_bert::pipelines::translation::{ Language, TranslationConfig, TranslationModel };

use tch::Device;
use tokio::{task, sync::Mutex };

use std::collections::HashMap;
use uuid::Uuid;

lazy_static! {
  pub static ref CONV_MANAGER: Mutex<ConversationManager>
    = Mutex::new(ConversationManager::new());
  pub static ref CHAT_CONTEXT: Mutex<HashMap<u64, (Uuid, u32)>>
    = Mutex::new(HashMap::new());
}

pub async fn en2ru(text: String) -> failure::Fallible<String> {
  task::spawn_blocking(move || {
    let translation_config =
      TranslationConfig::new(Language::EnglishToRussian, Device::cuda_if_available());
    let model = TranslationModel::new(translation_config)?;
    let output = model.translate(&[text.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig EnglishToRussian");
      Ok(text)
    } else {
      Ok(output[0].clone())
    }
  }).await.unwrap()
}

pub async fn ru2en(text: String) -> failure::Fallible<String> {
  task::spawn_blocking(move || {
    let translation_config =
      TranslationConfig::new(Language::RussianToEnglish, Device::cuda_if_available());
    let model = TranslationModel::new(translation_config)?;
    let output = model.translate(&[text.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(text)
    } else {
      let translation = &output[0];
      Ok(translation.clone())
    }
  }).await.unwrap()
}

pub async fn ru2en_many(texts: Vec<String>) -> failure::Fallible<Vec<String>> {
  task::spawn_blocking(move || {
    let ttt = texts.iter().map(|t| t.as_str()).collect::<Vec<&str>>();
    let translation_config =
      TranslationConfig::new(Language::RussianToEnglish, Device::cuda_if_available());
    let model = TranslationModel::new(translation_config)?;
    let output = model.translate(&ttt);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(Vec::new())
    } else {
      Ok(output)
    }
  }).await.unwrap()
}

fn ask_with_cache(q: String, cache: String) -> failure::Fallible<String> {
  // Set-up Question Answering model
  let qa_model = QuestionAnsweringModel::new(Default::default())?;

  let qa_input = QaInput {
    question: q,
    context: cache,
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

pub async fn ask(question: String) -> failure::Fallible<String> {
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let cache = 
    if cache_eng_vec.is_empty() {
      String::from("Humans imba")
    } else {
      cache_eng_vec.join(" ")
    };
  task::spawn_blocking(move || {
    ask_with_cache(question, cache)
  }).await.unwrap()
}

pub async fn chat(something: String, user_id: u64) -> failure::Fallible<String> {
  let mut conversation_manager = CONV_MANAGER.lock().await;
  let mut chat_context = CHAT_CONTEXT.lock().await;
  task::spawn_blocking(move || {
    let conversation_model = ConversationModel::new(Default::default())?;

    let output =
      if user_id != 0 {
        if let Some((tracking_conversation, passed)) = chat_context.get_mut(&user_id) {
          if let Some(found_conversation) = conversation_manager
                                        .get(tracking_conversation) {
            let _ = found_conversation.add_user_input(something.as_str());
            *passed = 0;
            conversation_model.generate_responses(&mut conversation_manager)
          } else {
            *tracking_conversation = conversation_manager.create(something.as_str());
            *passed = 0;
            conversation_model.generate_responses(&mut conversation_manager)
          }
        } else {
          chat_context.insert( user_id
                             , ( conversation_manager.create(something.as_str()), 0 )
                             );
          conversation_model.generate_responses(&mut conversation_manager)
        }
      } else {
        conversation_manager.create(something.as_str());
        conversation_model.generate_responses(&mut conversation_manager)
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
  }).await.unwrap()
}

#[cfg(test)]
mod bert_tests {
  use super::*;
  #[test]
  fn spell_test() -> Result<(), String> {
    let cache = String::from("Humba");
    match ask_with_cache(String::from("Humans imba?"), cache) {
      Ok(some) => {
        if !some.is_empty() {
          Ok(())
        } else {
          Err(String::from("empty output"))
        }
      }, Err(de) => Err(format!("Failed to get answer {:?}", de))
    }
  }
}

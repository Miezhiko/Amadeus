use crate::steins::ai::chain::CACHE_ENG_STR;

use rust_bert::pipelines::{
  conversation::{ ConversationManager, ConversationModel },
  question_answering::{ QaInput, QuestionAnsweringModel },
  translation::{ Language, TranslationConfig, TranslationModel }
};

use tch::Device;
use tokio::{ task, sync::Mutex };

use std::collections::HashMap;
use uuid::Uuid;

use eyre::Result;

lazy_static! {
  // models
  static ref DEVICE: Device = Device::cuda_if_available();
  pub static ref EN2RUMODEL: Mutex<TranslationModel> =
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::EnglishToRussian, *DEVICE)
    ).unwrap());
  pub static ref RU2ENMODEL: Mutex<TranslationModel> =
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::RussianToEnglish, *DEVICE)
    ).unwrap());
  pub static ref QAMODEL: Mutex<QuestionAnsweringModel> =
    Mutex::new(QuestionAnsweringModel::new(Default::default()).unwrap());
  pub static ref CONVMODEL: Mutex<ConversationModel> =
    Mutex::new(ConversationModel::new(Default::default()).unwrap());

  // chat context
  pub static ref CONV_MANAGER: Mutex<ConversationManager>
    = Mutex::new(ConversationManager::new());
  pub static ref CHAT_CONTEXT: Mutex<HashMap<u64, (Uuid, u32, u32)>>
    = Mutex::new(HashMap::new());
}

pub async fn en2ru(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let en2ru_model = EN2RUMODEL.lock().await;
  task::spawn_blocking(move || {
    let output = en2ru_model.translate(&[text.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig EnglishToRussian");
      Ok(text)
    } else {
      Ok(output[0].clone())
    }
  }).await.unwrap()
}

pub async fn ru2en(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let ru2en_model = EN2RUMODEL.lock().await;
  task::spawn_blocking(move || {
    let output = ru2en_model.translate(&[text.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(text)
    } else {
      let translation = &output[0];
      Ok(translation.clone())
    }
  }).await.unwrap()
}

pub async fn ru2en_many(texts: Vec<String>) -> Result<Vec<String>> {
  if texts.is_empty() {
    return Ok(vec![]);
  }
  let ru2en_model = EN2RUMODEL.lock().await;
  task::spawn_blocking(move || {
    let ttt = texts.iter().map(|t| t.as_str()).collect::<Vec<&str>>();
    let output = ru2en_model.translate(&ttt);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(Vec::new())
    } else {
      Ok(output)
    }
  }).await.unwrap()
}

pub async fn ask(question: String) -> Result<String> {
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let cache = 
    if cache_eng_vec.is_empty() {
      String::from("Humans imba")
    } else {
      cache_eng_vec.join(" ")
    };
  let qa_model = QAMODEL.lock().await;
  task::spawn_blocking(move || {
    let qa_input = QaInput {
      question: question,
      context: cache
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
  }).await.unwrap()
}

pub async fn chat(something: String, user_id: u64) -> Result<String> {
  let mut conversation_manager = CONV_MANAGER.lock().await;
  let mut chat_context = CHAT_CONTEXT.lock().await;
  let conversation_model = CONVMODEL.lock().await;
  task::spawn_blocking(move || {
    let output =
      if user_id != 0 {
        if let Some((tracking_conversation, passed, x)) = chat_context.get_mut(&user_id) {
          if *x > 5 {
            *tracking_conversation = conversation_manager.create(&something);
            *passed = 0; *x = 0;
            conversation_model.generate_responses(&mut conversation_manager)
          } else if let Some(found_conversation) = conversation_manager
                                             .get(tracking_conversation) {
            let _ = found_conversation.add_user_input(&something);
            *passed = 0; *x += 1;
            conversation_model.generate_responses(&mut conversation_manager)
          } else {
            *tracking_conversation = conversation_manager.create(&something);
            *passed = 0; *x = 0;
            conversation_model.generate_responses(&mut conversation_manager)
          }
        } else {
          chat_context.insert( user_id
                             , ( conversation_manager.create(&something), 0, 0 )
                             );
          conversation_model.generate_responses(&mut conversation_manager)
        }
      } else {
        conversation_manager.create(&something);
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

use crate::stains::ai::chain::CACHE_ENG_STR;

use rust_bert::pipelines::conversation::{ ConversationManager, ConversationModel };
use rust_bert::pipelines::question_answering::{ QaInput, QuestionAnsweringModel };
use rust_bert::pipelines::translation::{ Language, TranslationConfig, TranslationModel };

use tch::Device;

use failure;

use tokio::task;

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
      let translation = &output[0];
      Ok(translation.clone())
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

fn ask_with_cache(question: String, cache: String) -> failure::Fallible<String> {
  // Set-up Question Answering model
  let qa_model = QuestionAnsweringModel::new(Default::default())?;

  let qa_input = QaInput {
    question: question,
    context: cache,
  };

  // Get answer
  let answers = qa_model.predict(&vec![qa_input], 1, 32);
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

pub async fn chat(something: String) -> failure::Fallible<String> {
  task::spawn_blocking(move || {
    let conversation_model = ConversationModel::new(Default::default())?;
    let mut conversation_manager = ConversationManager::new();

    let _conversation_id = conversation_manager.create(something.as_str());

    let output = conversation_model.generate_responses(&mut conversation_manager);

    // TODO: follow onversation
    /*
    let _ = conversation_manager
        .get(&conversation_id)
        .unwrap()
        .add_user_input(something_else_str);
    let output = conversation_model.generate_responses(&mut conversation_manager);
    */

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

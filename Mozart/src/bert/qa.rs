use crate::{
  types::ChatResponse,
  cache::*,
  prelude::*,
  bert::{ GPT_LIMIT, LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use rust_bert::pipelines::{
  question_answering::{ QaInput
                      , QuestionAnsweringModel
                      , QuestionAnsweringConfig }
};

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use tokio::task;

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use rand::seq::SliceRandom;

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

async fn ask(msg_content: String, lsm: bool) -> anyhow::Result<String> {
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
        if let Some(answer) = my_answers.get(0) {
          if answer.answer.is_empty() || answer.answer.len() == 1 {
            Err(anyhow!("QA: bad answer, I don't like it"))
          } else {
            Ok(answer.answer.clone())
          }
        } else {
          Err(anyhow!("empty answer by GPT QA model"))
        }
      }
    } else {
      Err(anyhow!("Empty QA model"))
    }
  }).await.unwrap()
}

async fn ask_send( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> anyhow::Result<()> {
  match ask(something.clone(), lsm).await {
    Ok(result) => {
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      let package = ChatResponse {
        message: msg,
        channel: chan,
        response: result,
        russian
      };
      let encoded = bincode::encode_to_vec(&package, BINCODE_CONFIG)?;
      lukashenko.write_all(&encoded)?;
      Ok(())
    }, Err(why) => {
      error!("QA: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn ASK( msg: Option<u64>
                , chan: u64
                , something: String
                , user_id: u64
                , lsm: bool
                , russian: bool ) -> TaskResult<()> {
  if let Err(why) = ask_send(msg, chan, something, user_id, lsm, russian).await {
    error!("QA: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("ASK response sent to {LUKASHENKO}!");
    Ok(())
  }
}

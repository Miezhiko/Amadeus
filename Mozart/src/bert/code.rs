use crate::{
  types::ChatResponse,
  prelude::*,
  bert::{ LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::sequence_classification::{
    SequenceClassificationConfig, SequenceClassificationModel,
};
use rust_bert::resources::RemoteResource;
use rust_bert::roberta::{
    RobertaConfigResources, RobertaMergesResources, RobertaModelResources, RobertaVocabResources,
};

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use tokio::task;

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

fn codebert_model_loader() -> SequenceClassificationModel {
  let generate_config = SequenceClassificationConfig::new(
    ModelType::Roberta,
    RemoteResource::from_pretrained(RobertaModelResources::CODEBERTA_LANGUAGE_ID),
    RemoteResource::from_pretrained(RobertaConfigResources::CODEBERTA_LANGUAGE_ID),
    RemoteResource::from_pretrained(RobertaVocabResources::CODEBERTA_LANGUAGE_ID),
    Some(RemoteResource::from_pretrained(
        RobertaMergesResources::CODEBERTA_LANGUAGE_ID,
    )),
    false,
    None,
    None
  );
  SequenceClassificationModel::new(generate_config).unwrap()
}

static CODE_MODEL: Lazy<Mutex<Option<SequenceClassificationModel>>> =
  Lazy::new(|| Mutex::new(Some(codebert_model_loader())));

async fn codebert(msg_content: String, lsm: bool) -> anyhow::Result<String> {
  info!("Generating codebert response");
  let mut code_model = CODE_MODEL.lock().await;
  if code_model.is_none() {
    *code_model = Some( codebert_model_loader() );
  }
  let input_str = process_message_for_gpt(&msg_content);
  task::spawn_blocking(move || {
    if let Some(codem) = &mut *code_model {
      let output = codem.predict([input_str.as_str()]);
      if ! lsm {
        *code_model = None;
      }
      if output.is_empty() {
        Err(anyhow!("no output from codebert model"))
      } else {
        let answer = output[0].text.clone();
        if answer.is_empty() || answer.len() == 1 {
          Err(anyhow!("CodeBert: bad answer, I don't like it"))
        } else {
          Ok(answer)
        }
      }
    } else {
      Err(anyhow!("Empty codebert model"))
    }
  }).await.unwrap()
}

async fn code_send( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> anyhow::Result<()> {
  match codebert(something.clone(), lsm).await {
    Ok(result) => {
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      let package = ChatResponse {
        message: msg,
        channel: chan,
        response: result,
        russian: false
      };
      let encoded = bincode::encode_to_vec(package, BINCODE_CONFIG)?;
      lukashenko.write_all(&encoded)?;
      Ok(())
    }, Err(why) => {
      error!("codebert: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn CODEBERT( msg: Option<u64>
                     , chan: u64
                     , something: String
                     , user_id: u64
                     , lsm: bool
                     , russian: bool ) -> TaskResult<()> {
  if let Err(why) = code_send(msg, chan, something, user_id, lsm, russian).await {
    error!("codebert: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("codebert response sent to {LUKASHENKO}!");
    Ok(())
  }
}

use crate::{
  types::ChatResponse,
  cache::DEVICE,
  prelude::*,
  bert::{ LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use rust_bert::gpt_j::{ GptJConfigResources, GptJMergesResources, GptJVocabResources };
use rust_bert::pipelines::common::{ ModelType, ModelResource };
use rust_bert::pipelines::text_generation::{ TextGenerationConfig, TextGenerationModel };
use rust_bert::resources::{ LocalResource, RemoteResource };

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*,
  path::PathBuf
};

use tokio::task;

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

fn gptj_model_loader() -> TextGenerationModel {
  let config_resource = Box::new(RemoteResource::from_pretrained(
    GptJConfigResources::GPT_J_6B_FLOAT16,
  ));

  let vocab_resource = Box::new(RemoteResource::from_pretrained(
    GptJVocabResources::GPT_J_6B_FLOAT16,
  ));

  let merges_resource = Box::new(RemoteResource::from_pretrained(
    GptJMergesResources::GPT_J_6B_FLOAT16,
  ));

  let model_resource = Box::new(LocalResource::from(PathBuf::from(
    "resources/gpt-j-6B-float16/rust_model.ot",
  )));

  let generation_config = TextGenerationConfig {
      model_type: ModelType::GPTJ,
      model_resource: ModelResource::Torch(model_resource),
      config_resource,
      vocab_resource,
      merges_resource: Some(merges_resource),
      min_length: 10,
      max_length: Some(32),
      do_sample: false,
      early_stopping: true,
      num_beams: 1,
      num_return_sequences: 1,
      device: *DEVICE,
      ..Default::default()
  };

  TextGenerationModel::new(generation_config).unwrap()
}

static GPTJ_MODEL: Lazy<Mutex<Option<TextGenerationModel>>> =
  Lazy::new(|| Mutex::new(Some(gptj_model_loader())));

async fn gptj(msg_content: String, lsm: bool) -> anyhow::Result<String> {
  info!("Generating gptj response");
  let mut gptj_model = GPTJ_MODEL.lock().await;
  if gptj_model.is_none() {
    *gptj_model = Some( gptj_model_loader() );
  }
  let input_str = process_message_for_gpt(&msg_content);
  task::spawn_blocking(move || {
    if let Some(gptj_m) = &mut *gptj_model {
      let output = gptj_m.generate(&[input_str.as_str()], None);
      if ! lsm {
        *gptj_model = None;
      }
      if output.is_empty() {
        Err(anyhow!("no output from gptj model"))
      } else {
        let answer = output[0].clone();
        if answer.is_empty() || answer.len() == 1 {
          Err(anyhow!("GPTJ: bad answer, I don't like it"))
        } else {
          Ok(answer)
        }
      }
    } else {
      Err(anyhow!("Empty gptj model"))
    }
  }).await.unwrap()
}

async fn gptj_send( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> anyhow::Result<()> {
  match gptj(something.clone(), lsm).await {
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
      error!("gptj: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn GPTJ( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> TaskResult<()> {
  if let Err(why) = gptj_send(msg, chan, something, user_id, lsm, russian).await {
    error!("gptj: Failed to generate response, {why}");
  } else {
    info!("gptj response sent to {LUKASHENKO}!");
  }
  Ok(())
}

use crate::{
  types::ChatResponse,
  cache::*,
  prelude::*,
  bert::{ LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use rust_bert::pipelines::common::ModelType;
use rust_bert::pipelines::text_generation::{TextGenerationConfig, TextGenerationModel};
use rust_bert::resources::RemoteResource;
use rust_bert::xlnet::{XLNetConfigResources, XLNetModelResources, XLNetVocabResources};

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use tokio::task;

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use rand::seq::SliceRandom;

fn xlnet_model_loader() -> TextGenerationModel {
  let config_resource = Box::new(RemoteResource::from_pretrained(
    XLNetConfigResources::XLNET_BASE_CASED,
  ));
  let vocab_resource = Box::new(RemoteResource::from_pretrained(
    XLNetVocabResources::XLNET_BASE_CASED,
  ));
  let merges_resource = Box::new(RemoteResource::from_pretrained(
    XLNetVocabResources::XLNET_BASE_CASED,
  ));
  let model_resource = Box::new(RemoteResource::from_pretrained(
    XLNetModelResources::XLNET_BASE_CASED,
  ));
  let generate_config = TextGenerationConfig {
    model_type: ModelType::XLNet,
    model_resource,
    config_resource,
    vocab_resource,
    merges_resource: Some(merges_resource),
    max_length: Some(64),
    do_sample: false,
    num_beams: 3,
    temperature: 1.0,
    num_return_sequences: 1,
    ..Default::default()
  };
  TextGenerationModel::new(generate_config).unwrap()
}

static XLNET_MODEL: Lazy<Mutex<Option<TextGenerationModel>>> =
  Lazy::new(|| Mutex::new(Some(xlnet_model_loader())));

async fn xlnet(msg_content: String, lsm: bool) -> anyhow::Result<String> {
  info!("Generating xlnet response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let mut summ_model = XLNET_MODEL.lock().await;
  if summ_model.is_none() {
    *summ_model = Some( xlnet_model_loader() );
  }
  let input_str = process_message_for_gpt(&msg_content);
  let mut input =
    if cache_eng_vec.is_empty() { vec![] } else {
      cache_eng_vec.iter().collect::<Vec<&String>>()
        .choose_multiple(&mut rand::thread_rng(), 5)
        .map(|s| (*s).to_owned())
        .collect::<Vec<String>>()
    };
  input.push(input_str);
  task::spawn_blocking(move || {
    if let Some(sum) = &mut *summ_model {
      let output = sum.generate(&input, None);
      if ! lsm {
        *summ_model = None;
      }
      if output.is_empty() {
        Err(anyhow!("no output from xlnet model"))
      } else {
        let answer = output[0].clone();
        if answer.is_empty() || answer.len() == 1 {
          Err(anyhow!("XLNet: bad answer, I don't like it"))
        } else {
          Ok(answer)
        }
      }
    } else {
      Err(anyhow!("Empty xlnet model"))
    }
  }).await.unwrap()
}

async fn xlnet_send( msg: Option<u64>
                   , chan: u64
                   , something: String
                   , user_id: u64
                   , lsm: bool
                   , russian: bool ) -> anyhow::Result<()> {
  match xlnet(something.clone(), lsm).await {
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
      error!("xlnet: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn XLNET( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> TaskResult<()> {
  if let Err(why) = xlnet_send(msg, chan, something, user_id, lsm, russian).await {
    error!("xlnet: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("xlnet response sent to {LUKASHENKO}!");
    Ok(())
  }
}

use crate::{
  types::ChatResponse,
  cache::*,
  prelude::*,
  bert::{ LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use rust_bert::bart::{
  BartConfigResources, BartMergesResources, BartModelResources, BartVocabResources,
};
use rust_bert::pipelines::summarization::{ SummarizationConfig, SummarizationModel };
use rust_bert::resources::RemoteResource;

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use tokio::task;

use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use rand::seq::SliceRandom;

fn summarization_model_bart_loader() -> SummarizationModel {
  let config_resource = Box::new(RemoteResource::from_pretrained(
    BartConfigResources::DISTILBART_CNN_6_6,
  ));
  let vocab_resource = Box::new(RemoteResource::from_pretrained(
    BartVocabResources::DISTILBART_CNN_6_6,
  ));
  let merges_resource = Box::new(RemoteResource::from_pretrained(
    BartMergesResources::DISTILBART_CNN_6_6,
  ));
  let model_resource = Box::new(RemoteResource::from_pretrained(
    BartModelResources::DISTILBART_CNN_6_6,
  ));
  let summarization_config = SummarizationConfig {
    model_resource,
    config_resource,
    vocab_resource,
    merges_resource,
    num_beams: 1,
    length_penalty: 1.0,
    min_length: 56,
    max_length: 142,
    device: *DEVICE,
    ..Default::default()
  };
  SummarizationModel::new(summarization_config).unwrap()
}

static SUM_MODEL_BART: Lazy<Mutex<Option<SummarizationModel>>> =
  Lazy::new(|| Mutex::new(Some(summarization_model_bart_loader())));

async fn summarize(msg_content: String, lsm: bool) -> anyhow::Result<String> {
  info!("Generating Summarization response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let mut summ_model = SUM_MODEL_BART.lock().await;
  if summ_model.is_none() {
    *summ_model = Some( summarization_model_bart_loader() );
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
      let output = sum.summarize(&input);
      if ! lsm {
        *summ_model = None;
      }
      if output.is_empty() {
        Err(anyhow!("no output from Summarization model"))
      } else {
        let answer = output[0].clone();
        if answer.is_empty() || answer.len() == 1 {
          Err(anyhow!("Summarization: bad answer, I don't like it"))
        } else {
          Ok(answer)
        }
      }
    } else {
      Err(anyhow!("Empty Summarization model"))
    }
  }).await.unwrap()
}

async fn summarize_send( msg: Option<u64>
                       , chan: u64
                       , something: String
                       , user_id: u64
                       , lsm: bool
                       , russian: bool ) -> anyhow::Result<()> {
  match summarize(something.clone(), lsm).await {
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
      error!("Summarization: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn SUMMARIZE( msg: Option<u64>
                      , chan: u64
                      , something: String
                      , user_id: u64
                      , lsm: bool
                      , russian: bool ) -> TaskResult<()> {
  if let Err(why) = summarize_send(msg, chan, something, user_id, lsm, russian).await {
    error!("Summarization: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("Summarization response sent to {LUKASHENKO}!");
    Ok(())
  }
}

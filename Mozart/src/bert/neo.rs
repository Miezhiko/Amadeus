use crate::{
  types::ChatResponse,
  bert::{ process_message_for_gpt, LUKASHENKO
        , chat::chat_gpt2_send },
  cache::{ DEVICE, CACHE_ENG_STR },
  prelude::*
};

use celery::prelude::*;

use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::{
  pipelines::common::{ ModelType, ModelResource },
  pipelines::text_generation::{ TextGenerationConfig, TextGenerationModel },
  resources::RemoteResource
};

use once_cell::sync::Lazy;
use tokio::{ task, sync::Mutex };

use rand::seq::SliceRandom;

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

fn neo_model_loader() -> TextGenerationModel {
  let config_resource = Box::new(RemoteResource::from_pretrained(
    GptNeoConfigResources::GPT_NEO_125M,
  ));
  let vocab_resource = Box::new(RemoteResource::from_pretrained(
    GptNeoVocabResources::GPT_NEO_125M,
  ));
  let merges_resource = Box::new(RemoteResource::from_pretrained(
    GptNeoMergesResources::GPT_NEO_125M,
  ));
  let model_resource = Box::new(RemoteResource::from_pretrained(
    GptNeoModelResources::GPT_NEO_125M,
  ));
  let generate_config = TextGenerationConfig {
    model_type: ModelType::GPTNeo,
    model_resource: ModelResource::Torch(model_resource),
    config_resource,
    vocab_resource,
    merges_resource: Some(merges_resource),
    min_length: 10,
    max_length: Some(30),
    do_sample: true,
    early_stopping: true,
    num_beams: 4,
    num_return_sequences: 1,
    device: *DEVICE,
    ..Default::default()
  };
  TextGenerationModel::new(generate_config).unwrap()
}

static NEOMODEL: Lazy<Mutex<Option<TextGenerationModel>>> =
  Lazy::new(||{ Mutex::new(Some(neo_model_loader())) });

static NEO_SEPARATORS: [char; 3] = ['"', '*', '”'];
static A: &str = "A: ";

async fn chat_neo(something: String, lsm: bool) -> anyhow::Result<String> {
  info!("Generating GPT Neo response");

  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let mut neo = NEOMODEL.lock().await;
  if neo.is_none() {
    *neo = Some(neo_model_loader());
  }
  let cache_vec = cache_eng_vec.iter().collect::<Vec<&String>>();
  let mut cache_slices = cache_vec
                        .choose_multiple(&mut rand::thread_rng(), 32)
                        .map(AsRef::as_ref).collect::<Vec<&str>>();
  cache_slices.push(&something);

  let neo_result =
    task::spawn_blocking(move || {
      if let Some(neo_model) = &mut *neo {
        let output = neo_model.generate(&[something.as_str()], None);
        let result = if output.is_empty() {
            error!("Failed to chat with Neo Model");
            Err(anyhow!("no output from GPT neo model"))
          } else { Ok({
            if output.len() > 1 {
              if let Some(r) = output.choose(&mut rand::thread_rng()) {
                if r.contains("following code:") {
                  output[0].clone()
                } else {
                  String::from(r)
                }
              } else {
                output[1].clone()
              }
            } else {
              output[0].clone()
            } })
          };
        if !lsm {
          *neo = None;
        }
        result
      } else {
        Err(anyhow!("Empty GPT Neo model"))
      }
    }).await.unwrap()?;

  let split = neo_result.split(&NEO_SEPARATORS[..]).collect::<Vec<&str>>();
  if let Some(first) = split.choose(&mut rand::thread_rng()) {
    let result = process_message_for_gpt(first);
    if result.is_empty() || result.len() == 1 {
      Err( anyhow!("only trash in chat neo response") )
    } else if result.chars().any(|c| matches!(c, '$' | '{' | '}'))
           || result.contains("following code") {
      Err( anyhow!("BAD RESULT") )
    } else if result.contains(A) {
      let a_split = result.split(A).collect::<Vec<&str>>();
      if a_split.len() > 1 {
        Ok( a_split[1].to_string() )
      } else {
        Ok( result.replace(A, "") )
      }
    } else {
      Ok( result )
    }
  } else {
    Err( anyhow!("output was literally only quotes >_<") )
  }
}

async fn chat_neo_send( msg: Option<u64>
                      , chan: u64
                      , something: String
                      , user_id: u64
                      , lsm: bool
                      , russian: bool ) -> anyhow::Result<()> {
  match chat_neo(something.clone(), lsm).await {
    Ok(result) => {
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      let package = ChatResponse {
        message: msg,
        channel: chan,
        response: result,
        russian
      };
      let encoded = bincode::encode_to_vec(package, BINCODE_CONFIG)?;
      lukashenko.write_all(&encoded)?;
      Ok(())
    }, Err(why) => {
      error!("NEO: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn CHAT_NEO( msg: Option<u64>
                     , chan: u64
                     , something: String
                     , user_id: u64
                     , lsm: bool
                     , russian: bool ) -> TaskResult<()> {
  if let Err(why) = chat_neo_send(msg, chan, something, user_id, lsm, russian).await {
    error!("Failed to generate NEO response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("NEO: response sent to {LUKASHENKO}!");
    Ok(())
  }
}

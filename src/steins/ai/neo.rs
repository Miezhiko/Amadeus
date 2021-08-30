use crate::steins::ai::cache::{
  process_message_for_gpt,
  CACHE_ENG_STR
};

use rust_bert::gpt_neo::{
    GptNeoConfigResources, GptNeoMergesResources, GptNeoModelResources, GptNeoVocabResources,
};
use rust_bert::{
  pipelines::common::ModelType,
  pipelines::text_generation::{TextGenerationConfig, TextGenerationModel},
  resources::{RemoteResource, Resource}
};
use tch::Device;

use once_cell::sync::Lazy;
use tokio::{ task, sync::Mutex };

use rand::seq::SliceRandom;

pub static NEOMODEL: Lazy<Mutex<TextGenerationModel>> =
  Lazy::new(||{
   let config_resource = Resource::Remote(RemoteResource::from_pretrained(
      GptNeoConfigResources::GPT_NEO_1_3B,
    ));
    let vocab_resource = Resource::Remote(RemoteResource::from_pretrained(
      GptNeoVocabResources::GPT_NEO_1_3B,
    ));
    let merges_resource = Resource::Remote(RemoteResource::from_pretrained(
      GptNeoMergesResources::GPT_NEO_1_3B,
    ));
    let model_resource = Resource::Remote(RemoteResource::from_pretrained(
      GptNeoModelResources::GPT_NEO_1_3B,
    ));
    let generate_config = TextGenerationConfig {
      model_type: ModelType::GPTNeo,
      model_resource,
      config_resource,
      vocab_resource,
      merges_resource,
      min_length: 10,
      max_length: 64,
      do_sample: false,
      early_stopping: true,
      num_beams: 4,
      num_return_sequences: 1,
      device: Device::Cpu,
      ..Default::default()
    };
    Mutex::new( TextGenerationModel::new(generate_config).unwrap() )
  });

static NEO_SEPARATORS: [char; 3] = ['"', '*', '”'];

pub async fn chat_neo(something: String) -> anyhow::Result<String> {
  info!("Generating GPT Neo response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let neo_model = NEOMODEL.lock().await;
  let cache_vec = cache_eng_vec.iter().collect::<Vec<&String>>();
  let mut cache_slices = cache_vec
                        .choose_multiple(&mut rand::thread_rng(), 32)
                        .map(AsRef::as_ref).collect::<Vec<&str>>();
  cache_slices.push(&something);

  let neo_result =
    task::spawn_blocking(move || {
      let output = neo_model.generate(&[something.as_str()], None);
      if output.is_empty() {
        error!("Failed to chat with Neo Model");
        Err(anyhow!("no output from GPT neo model"))
      } else { Ok(
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
        } )
      }
    }).await.unwrap()?;

  let split = neo_result.split(&NEO_SEPARATORS[..]).collect::<Vec<&str>>();
  if let Some(first) = split.choose(&mut rand::thread_rng()) {
    let result = process_message_for_gpt(first);
    if result.is_empty() {
      Err( anyhow!("only trash in chat neo response") )
    } else {
      if result.contains("following code:") {
        Err( anyhow!("BAD RESULT") )
      } else {
        Ok( result )
      }
    }
  } else {
    Err( anyhow!("output was literally only quotes >_<") )
  }
}

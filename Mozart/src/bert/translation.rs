use rust_bert::pipelines::{
  translation::{ Language
               , TranslationModelBuilder
               , TranslationModel }
};

use tch::Device;
use tokio::task;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

use chrono::{ Utc, DateTime };

pub static TRANSLATION_LIMIT: usize = 512;

fn enru_model_loader() -> TranslationModel {
  TranslationModelBuilder::new()
    .with_source_languages(vec![Language::English, Language::Russian])
    .with_target_languages(vec![Language::English, Language::Russian])
    .with_device(Device::cuda_if_available())
    .create_model().unwrap()
}

pub static ENRUMODEL: Lazy<Mutex<Option<TranslationModel>>> =
  Lazy::new(|| Mutex::new( Some( enru_model_loader() ) ) );

pub static ENRUMODEL_USED: Lazy<Mutex<Option<DateTime<Utc>>>> =
  Lazy::new(|| Mutex::new(None));

pub async fn en2ru(text: String, lsm: bool) -> anyhow::Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let mut enru_model = ENRUMODEL.lock().await;
  if enru_model.is_none() {
    *enru_model = Some( enru_model_loader() );
  }
  if !lsm {
    let mut enru_model_model_used = ENRUMODEL_USED.lock().await;
    *enru_model_model_used = Some(Utc::now());
  }
  task::spawn_blocking(move || {
    if let Some(en2ru_model) = &mut *enru_model {
      let mut something = text;
      if something.len() > TRANSLATION_LIMIT {
        if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
          something = something[i..].to_string();
        }
      }
      let output = en2ru_model.translate(&[something.as_str()], Some(Language::English)
                                                              , Language::Russian)?;
      if output.is_empty() {
        Ok(something)
      } else {
        Ok(output[0].clone())
      }
    } else {
      Err(anyhow!("Empty ENRU model"))
    }
  }).await.unwrap()
}

pub async fn ru2en(text: String, lsm: bool) -> anyhow::Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let mut enru_model = ENRUMODEL.lock().await;
  if enru_model.is_none() {
    *enru_model = Some( enru_model_loader() );
  }
  if !lsm {
    let mut enru_model_model_used = ENRUMODEL_USED.lock().await;
    *enru_model_model_used = Some(Utc::now());
  }
  task::spawn_blocking(move || {
    if let Some(ru2en_model) = &mut *enru_model {
      let mut something = text;
      if something.len() > TRANSLATION_LIMIT {
        if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
          something = something[i..].to_string();
        }
      }
      let output = ru2en_model.translate(&[something.as_str()], Some(Language::Russian)
                                                              , Language::English)?;
      if output.is_empty() {
        Ok(something)
      } else {
        let translation = &output[0];
        Ok(translation.clone())
      }
    } else {
      Err(anyhow!("Empty ENRU model"))
    }
  }).await.unwrap()
}

#[derive(Debug)]
pub enum SLanguage {
  English,
  Russian,
  Ukrainian,
  German,
  French
}

fn from_slang(sl: &SLanguage) -> Language {
  match sl {
    SLanguage::English    => Language::English,
    SLanguage::Russian    => Language::Russian,
    SLanguage::Ukrainian  => Language::Ukrainian,
    SLanguage::German     => Language::German,
    SLanguage::French     => Language::French
  }
}

pub async fn bert_translate( text: String
                           , source_slang: SLanguage
                           , target_slang: SLanguage ) -> anyhow::Result<String> {
  let source_lang = from_slang(&source_slang);
  let target_lang = from_slang(&target_slang);
  static RUEN_LANGS: &[Language; 2] = &[Language::Russian, Language::English];
  if RUEN_LANGS.contains(&source_lang) && RUEN_LANGS.contains(&target_lang) {
    if source_lang == Language::Russian {
      ru2en(text, true).await
    } else {
      en2ru(text, true).await
    }
  } else {
    task::spawn_blocking(move || {
      let mut something = text;
      if something.len() > TRANSLATION_LIMIT {
        if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
          something = something[i..].to_string();
        }
      }

      let model = TranslationModelBuilder::new()
          .with_source_languages(vec![source_lang])
          .with_target_languages(vec![target_lang])
          .with_device(tch::Device::cuda_if_available())
          .create_model()?;

      let output = model.translate(&[something.as_str()], Some(source_lang), target_lang)?;

      if output.is_empty() {
        Err(anyhow!("Failed to translate with TranslationConfig EnglishToRussian"))
      } else {
        let translation = &output.join(" ");
        Ok(translation.clone())
      }
    }).await?
  }
}

/* unused for now

async fn en2ru_send( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , lsm: bool ) -> anyhow::Result<()> {
  let result = en2ru(something, lsm).await?;
  let temp_dir = std::env::temp_dir();
  let mut lukashenko = UnixStream::connect(temp_dir.join(TRANSLATION))?;
  let package = crate::types::ChatResponse {
    message: msg,
    channel: chan,
    response: result
  };
  let encoded = bincode::encode_to_vec(&package, BINCODE_CONFIG)?;
  lukashenko.write_all(&encoded)?;
  Ok(())
}

async fn ru2en_send( msg: Option<u64>
                   , chan: u64
                   , something: String
                   , lsm: bool ) -> anyhow::Result<()> {
  let result = ru2en(something, lsm).await?;
  let temp_dir = std::env::temp_dir();
  let mut lukashenko = UnixStream::connect(temp_dir.join(TRANSLATION))?;
  let package = crate::types::ChatResponse {
    message: msg,
    channel: chan,
    response: result
  };
  let encoded = bincode::encode_to_vec(&package, BINCODE_CONFIG)?;
  lukashenko.write_all(&encoded)?;
  Ok(())
}

#[celery::task]
pub async fn RU2EN( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , lsm: bool ) -> TaskResult<()> {
  if let Err(why) = ru2en_send(msg, chan, something, lsm).await {
    error!("Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("RU2EN response sent to {TRANSLATION}!");
    Ok(())
  }
}

#[celery::task]
pub async fn EN2RU( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , lsm: bool ) -> TaskResult<()> {
  if let Err(why) = en2ru_send(msg, chan, something, lsm).await {
    error!("Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("EN2RU response sent to {TRANSLATION}!");
    Ok(())
  }
}

*/
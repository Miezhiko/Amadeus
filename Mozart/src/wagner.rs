use crate::{
  types::ChatResponse,
  prelude::*,
  bert::{ LUKASHENKO
        , process_message_for_gpt
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

use wagner::{
  gpt4free,
  opengpt,
  poe
};

async fn wagner(msg_content: &str) -> anyhow::Result<String> {
  info!("Generating vivaldi response");
  let payload = process_message_for_gpt(msg_content);
  let mut fmode = true;
  if payload.contains("please") || payload.contains("пожалуйста") {
    fmode = false;
  } else if payload.contains("Please")
         || payload.contains("Пожалуйста")
         || payload.contains("PLEASE") {
    if let Ok(gpt4free_result) = poe::generate( &payload ) {
      return Ok(gpt4free_result)
    } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( &payload ) {
      return Ok(gpt4free_result)
    }
    fmode = false;
  }

  if let Ok(gpt4free_result)        = gpt4free::useless::generate( &payload, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::aiassist::generate( &payload, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::deepai::generate( &payload, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::gptworldAi::generate( &payload, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( &payload ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::italygpt::generate( &payload, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( &payload ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::theb::generate( &payload ) {
    Ok(gpt4free_result)
  } else { Err(anyhow!("Failed to generate Wagner response")) }
}

async fn wagner_send( msg: Option<u64>
                    , chan: u64
                    , something: String
                    , user_id: u64
                    , lsm: bool
                    , russian: bool ) -> anyhow::Result<()> {
  match wagner(something.as_str()).await {
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
      error!("wagner: Failed to generate response: {why}, using fallback to GPT2");
      chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
    }
  }
}

#[celery::task]
pub async fn WAGNER( msg: Option<u64>
                     , chan: u64
                     , something: String
                     , user_id: u64
                     , lsm: bool
                     , russian: bool ) -> TaskResult<()> {
  if let Err(why) = wagner_send(msg, chan, something, user_id, lsm, russian).await {
    error!("wagner: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("wagner response sent to {LUKASHENKO}!");
    Ok(())
  }
}

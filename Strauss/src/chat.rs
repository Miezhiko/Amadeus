use crate::{
  types::ChatResponse,
  prelude::*,
  bert::{ LUKASHENKO
        , chat::chat_gpt2_send }
};

use celery::prelude::*;

use std::{ 
  os::unix::net::UnixStream,
  io::prelude::*
};

async fn chat_send( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> anyhow::Result<()> {
  if user_id == 510368731378089984 {
    if something.contains("MULTIGEN") {
      let payload = something.replace("MULTIGEN ", "")
                             .replace("MULTIGEN", "");
      let resp_list = chat::generate_all(&payload, "Amadeus", false).await;
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      for (model_name, resp) in resp_list {
        let resp_format = match resp {
          Ok(r) => r,
          Err(err) => format!("Failed: {err}")
        };
        let model_response = format!("**{model_name}**:\n{resp_format}");
        let package = ChatResponse {
          message: msg,
          channel: chan,
          response: model_response,
          russian: false
        };
        let encoded = bincode::encode_to_vec(package, BINCODE_CONFIG)?;
        lukashenko.write_all(&encoded)?;
      }
      Ok(())
    } else {
      let response = match chat::generate(&something, "Amadeus", false).await {
        Ok(resp) => resp,
        Err(why) => format!("Failed: {why}")
      };
      let temp_dir = std::env::temp_dir();
      let mut lukashenko = UnixStream::connect(temp_dir.join(LUKASHENKO))?;
      let package = ChatResponse {
        message: msg,
        channel: chan,
        response: response,
        russian: false
      };
      let encoded = bincode::encode_to_vec(package, BINCODE_CONFIG)?;
      lukashenko.write_all(&encoded)?;
      Ok(())
    }
  } else {
    match chat::chat(something.as_str(), "Amadeus").await {
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
        error!("chat: Failed to generate response: {why}, using fallback to GPT2");
        chat_gpt2_send(msg, chan, something, user_id, lsm, russian, 0).await
      }
    }
  }
}

#[celery::task]
pub async fn CHAT( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , lsm: bool
                 , russian: bool ) -> TaskResult<()> {
  if let Err(why) = chat_send(msg, chan, something, user_id, lsm, russian).await {
    error!("chat: Failed to generate response, {why}");
    Err( TaskError::ExpectedError( why.to_string() ) )
  } else {
    info!("chat response sent to {LUKASHENKO}!");
    Ok(())
  }
}

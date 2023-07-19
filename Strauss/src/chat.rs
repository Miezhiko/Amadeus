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

async fn chat(msg_content: &str, user_id: u64) -> anyhow::Result<String> {
  info!("Generating chat response");
  let payload = process_message_for_gpt(msg_content);
  if user_id == 510368731378089984 {
    chat::generate(&payload).await
  } else {
    chat::chat(&payload, "Amadeus").await
  }
}

async fn chat_send( msg: Option<u64>
                  , chan: u64
                  , something: String
                  , user_id: u64
                  , lsm: bool
                  , russian: bool ) -> anyhow::Result<()> {
  match chat(something.as_str(), user_id).await {
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

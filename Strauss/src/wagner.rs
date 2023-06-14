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

async fn wagner(msg_content: &str) -> anyhow::Result<String> {
  info!("Generating Wagner response");
  let payload = process_message_for_gpt(msg_content);
  wagner::wagner(&payload).await
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

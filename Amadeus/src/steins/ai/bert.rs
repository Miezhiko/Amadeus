use crate::salieri::SALIERI;

use strauss::chat::CHAT;

use anyhow::Result;
use rand::Rng;

async fn salieri_request<T>( sig: celery::task::Signature<T>
                           ) -> Result<Option<String>>
                           where T: celery::task::Task {
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(sig).await?;
    Ok(None)
  } else {
    Err(anyhow!("BERT: Failed to connecto to Salieri"))
  }
}

pub async fn chat( msg: Option<u64>
                 , chan: u64
                 , something: String
                 , user_id: u64
                 , guild_id: u64 ) -> Result<Option<String>> {
  salieri_request(CHAT::new(msg, chan, something, user_id)).await
}

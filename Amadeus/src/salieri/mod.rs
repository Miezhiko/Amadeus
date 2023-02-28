pub mod handler;

use crate::{
  salieri::handler::{ handle_salieri
                    , handle_lukashenko },
};

use std::sync::Arc;
use async_std::fs;

use tokio::{ sync::Mutex, select
           , net::UnixListener
};

use once_cell::sync::Lazy;
use celery::Celery;

use mozart::{
  commands::SALIERI_SOCKET,
  bert::LUKASHENKO
};

use serenity::prelude::*;

type SalieriBroker = Arc<Celery>;

pub static SALIERI: Lazy<Mutex<Option<SalieriBroker>>> =
  Lazy::new(|| Mutex::new(None));

async fn process_salieri(ctx: &Context, salieri_socket: &UnixListener) {
  match salieri_socket.accept().await {
    Ok((stream, _addr)) => {
      if let Err(err) = handle_salieri(ctx, stream).await {
        error!("Failed to handle salieri client {err}");
      }
    }
    Err(e) => error!("{SALIERI_SOCKET} connection failed {e}")
  }
}

async fn process_lukashenko(ctx: &Context, lukashenko: &UnixListener) {
  match lukashenko.accept().await {
    Ok((stream, _addr)) => {
      if let Err(err) = handle_lukashenko(ctx, stream).await {
        error!("Failed to handle lukashenko client {err}");
      }
    }
    Err(e) => error!("{LUKASHENKO} connection failed {e}")
  }
}

pub async fn salieri_init(ctx: &Arc<Context>) -> anyhow::Result<()> {
  match mozart::celery_init(mozart::SALIERI_AMPQ).await {
    Ok(c) => {
      let mut salieri_lock_mut = SALIERI.lock().await;
      *salieri_lock_mut = Some(c);
    },
    Err(why) => {
      error!("failed to connect to Salieri services: {why}");
    }
  }
  let salieri_lock = SALIERI.lock().await;
  if let Some(salieri) = &*salieri_lock {
    salieri.send_task(mozart::AMADEUS_INIT::new()).await?;

    let temp_dir = std::env::temp_dir();

    set!{ salieri_address     = temp_dir.join(SALIERI_SOCKET)
        , lukashenko_address  = temp_dir.join(LUKASHENKO) };

    if salieri_address.as_path().exists() {
      fs::remove_file(&salieri_address).await?;
    }
    if lukashenko_address.as_path().exists() {
      fs::remove_file(&lukashenko_address).await?;
    }

    set!{ salieri_socket  = UnixListener::bind(salieri_address)?
        , lukashenko      = UnixListener::bind(lukashenko_address)? };

    let ctx_clone = Arc::clone(ctx);

    tokio::spawn(async move {
      loop {
        select! {
          _ = process_salieri(&ctx_clone, &salieri_socket) => {},
          _ = process_lukashenko(&ctx_clone, &lukashenko) => {}
        }
      }
    });
  }
  Ok(())
}

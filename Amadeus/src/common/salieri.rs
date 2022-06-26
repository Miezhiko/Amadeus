use crate::{
  common::msg::reply,
  steins::ai::cache::KATHOEY
};

use std::{
  sync::Arc, io
};
use async_std::fs;

use tokio::{ sync::Mutex, select
           , net::{ UnixListener, UnixStream }
};

use once_cell::sync::Lazy;
use celery::{ Celery, broker::AMQPBroker };

use rand::Rng;

use mozart::{
  types::ChatResponse,
  commands::{ SALIERI_SOCKET, self },
  bert::LUKASHENKO,
  prelude::*
};

use serenity::{
  prelude::*,
  model::id::{ ChannelId, MessageId }
};

type SalieriBroker = Arc<Celery<AMQPBroker>>;

pub static SALIERI: Lazy<Mutex<Option<SalieriBroker>>> =
  Lazy::new(|| Mutex::new(None));

async fn handle_salieri(_ctx: &Context, stream: UnixStream) -> anyhow::Result<()> {
  loop {
    stream.readable().await?;

    let mut buf = Vec::with_capacity(4096);
    match stream.try_read_buf(&mut buf) {
      Ok(0) => break,
      Ok(n) => {
        info!("read {} bytes", n);
      }
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        continue;
      }
      Err(e) => {
        return Err(e.into());
      }
    }

    let s = String::from_utf8(buf)?;
    match s.as_str() {
      commands::ERROR_HANDLE => (),
      commands::RESTART_HANDLE => (),
      _ => ()
    }
  }
  Ok(())
}

async fn handle_lukashenko(ctx: &Context, stream: UnixStream) -> anyhow::Result<()> {
  loop {
    stream.readable().await?;

    let mut buf = Vec::with_capacity(4096);
    match stream.try_read_buf(&mut buf) {
      Ok(0) => break,
      Ok(n) => {
        info!("read {} bytes", n);
      }
      Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        continue;
      }
      Err(e) => {
        return Err(e.into());
      }
    }

    let (decoded, _len): (ChatResponse, usize) = bincode::decode_from_slice(&buf[..], BINCODE_CONFIG)?;
    let chan: ChannelId = ChannelId( to_nzu!( decoded.channel ) );
    let response: String;
    if decoded.russian {
      match mozart::bert::translation::en2ru(decoded.response.clone()).await {
        Ok(translated) => {
          let rnda: u32 = rand::thread_rng().gen_range(0..10);
          if rnda != 1 {
            let kathoey = KATHOEY.lock().await;
            let rndy: u32 = rand::thread_rng().gen_range(0..30);
            response =
              if rndy == 1 {
                kathoey.extreme_feminize(&translated)
              } else {
                kathoey.feminize(&translated)
              };
          } else {
            response = translated;
          }
        } Err(why) => {
          error!("failed to translate salieri text to russian, {why}");
          response = decoded.response;
        }
      }
    } else {
      response = decoded.response;
    }

    if let Some(msg_id) = &decoded.message {
      if let Ok(msg) = chan.message(ctx, MessageId( to_nzu!( *msg_id ) )).await {
        reply(ctx, &msg, &response).await;
      }
    } else {
      chan.send_message(ctx, |m| m.content(&response)).await?;
    }
  }
  Ok(())
}

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

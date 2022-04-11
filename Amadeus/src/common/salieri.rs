use crate::common::msg::reply;

use std::{
  sync::Arc,
  io, fs
};

use tokio::{ sync::Mutex, select
           , net::{ UnixListener, UnixStream }
};

use once_cell::sync::Lazy;
use celery::{ Celery, broker::AMQPBroker };

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
      commands::GET_CACHE => (),
      commands::SET_CACHE => (),
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
    let chan: ChannelId = ChannelId( decoded.channel );
    if let Some(msg_id) = &decoded.message {
      if let Ok(msg) = chan.message(ctx, MessageId( *msg_id )).await {
        reply(ctx, &msg, &decoded.response).await;
      }
    } else {
      chan.send_message(ctx, |m| m.content(&decoded.response)).await?;
    }
  }
  Ok(())
}

/*
async fn handle_translation(ctx: &Context, stream: UnixStream) -> anyhow::Result<()> {
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
    let chan: ChannelId = ChannelId( decoded.channel );
    if let Some(msg_id) = &decoded.message {
      if let Ok(msg) = chan.message(ctx, MessageId( *msg_id )).await {
        reply(ctx, &msg, &decoded.response).await;
      }
    } else {
      chan.send_message(ctx, |m| m.content(&decoded.response)).await?;
    }
  }
  Ok(())
}
*/

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

/*
async fn process_translation(ctx: &Context, translation: &UnixListener) {
  match translation.accept().await {
    Ok((stream, _addr)) => {
      if let Err(err) = handle_translation(ctx, stream).await {
        error!("Failed to handle translation client {err}");
      }
    }
    Err(e) => error!("{TRANSLATION} connection failed {e}")
  }
}
*/

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
        //, translation_address = temp_dir.join(TRANSLATION) };

    fs::remove_file(&salieri_address)?;
    fs::remove_file(&lukashenko_address)?;
    //fs::remove_file(&translation_address)?;

    set!{ salieri_socket  = UnixListener::bind(salieri_address)?
        , lukashenko      = UnixListener::bind(lukashenko_address)? };
        //, translation     = UnixListener::bind(translation_address)? };

    let ctx_clone = Arc::clone(ctx);

    tokio::spawn(async move {
      loop {
        select! {
          _ = process_salieri(&ctx_clone, &salieri_socket) => {},
          _ = process_lukashenko(&ctx_clone, &lukashenko) => {}
          //_ = process_translation(&ctx_clone, &translation) => {},
        }
      }
    });
  }
  Ok(())
}

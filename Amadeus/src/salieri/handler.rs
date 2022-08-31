use crate::{
  common::msg::reply,
  steins::ai::cache::KATHOEY,
};

use mozart::{
  types::ChatResponse,
  commands,
  prelude::*
};

use std::io;
use rand::Rng;

use serenity::{
  prelude::*,
  builder::CreateMessage,
  model::id::{ ChannelId, MessageId }
};

use tokio::net::UnixStream;

pub async fn handle_salieri(_ctx: &Context, stream: UnixStream) -> anyhow::Result<()> {
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

pub async fn handle_lukashenko(ctx: &Context, stream: UnixStream) -> anyhow::Result<()> {
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
      chan.send_message(ctx, CreateMessage::new().content(&response)).await?;
    }
  }
  Ok(())
}

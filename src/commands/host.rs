use crate::{
  types::common::PubCreds,
  common::{ msg::channel_message
          , msg::direct_message
          , help::members::get_player }
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use flo_grpc::Channel;
use flo_grpc::controller::*;
use flo_grpc::player::*;

//use flo_grpc::game::*;

//use tonic::Request;
//const MAP: &str = r#"maps\frozenthrone\(4)twistedmeadows.w3x"#;

async fn get_grpc_client(flo_secret: String) -> flo_controller_client::FloControllerClient<Channel> {
  let channel = Channel::from_static("tcp://service.w3flo.com:3549")
    .connect()
    .await
    .unwrap();
  flo_controller_client::FloControllerClient::with_interceptor(
    channel, move |mut req: tonic::Request<()>| {
      req.metadata_mut()
         .insert("x-flo-secret", flo_secret.parse().unwrap());
    Ok(req)
  })
}

#[command]
async fn flo_nodes(ctx: &Context, msg: &Message) -> CommandResult {
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;
  let nodes = rpc.list_nodes(()).await;
  channel_message(ctx, msg, &format!("{:?}", nodes)).await;
  Ok(())
}

#[command]
#[min_args(1)]
#[owners_only]
async fn register_player(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;
  let res = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: 2, // PlayerSource::Api as i32,
      name: meme.user.name,
      source_id: meme.user.id.0.to_string(),
      ..Default::default()
    }).await?.into_inner();

  direct_message(ctx, msg, &format!("token {}", res.token)).await;

  if let Some(p) = res.player {
    channel_message(ctx, msg, &format!("registered {}, token sent via DM", p.id)).await;
  } else {
    channel_message(ctx, msg, "token sent via DM").await;
  }
  Ok(())
}

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
use flo_grpc::game::*;

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
      source: PlayerSource::Api as i32,
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

fn get_map() -> eyre::Result<Map> {
  let map = Map {
    sha1: hex::decode("9524abb8e35ce7b158bfa4d4b8734234d6073ca5")?,
    checksum: 3851316688u32,
    name: "Amadeus".to_string(),
    description: "The Global Warming cannot be stopped and the last survivors turnout back to the upper Lands behind. Now, even the last dry lands are flooding and the last remainings are fighting for it.".to_string(),
    author: "OmGan, edit by ESL".to_string(),
    path: "maps/frozenthrone/community/(2)lastrefuge.w3x".to_string(),
    width: 84,
    height: 84,
    players: vec![
      MapPlayer { name: "Player 1".to_string(), r#type: 1, flags: 0, ..Default::default() },
      MapPlayer { name: "Player 2".to_string(), r#type: 1, flags: 0, ..Default::default() }
    ],
    forces: vec![
      MapForce { name: "Force 1".to_string(), flags: 0, player_set: 4294967295, ..Default::default() }
    ]
  };
  Ok(map)
}

#[command]
#[min_args(1)]
#[owners_only]
async fn create_game(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
  //let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;

  let players = vec![316, 317];

  let res = rpc
    .create_game_as_bot(CreateGameAsBotRequest {
      name: "TEST".to_string(),
      map: Some(get_map()?),
      node_id: 14, //russia
      slots: players
        .into_iter()
        .enumerate()
        .map(|(idx, id)| CreateGameSlot {
          player_id: Some(id),
          settings: SlotSettings {
            team: idx as i32,
            color: idx as i32,
            status: 2,
            handicap: 100,
            ..Default::default()
          }
          .into(),
        })
        .collect(),
      ..Default::default()
    })
    .await?;

  channel_message(ctx, msg, &format!("Game created, id: {}"
                 , res.into_inner().game.unwrap().id)).await;
  Ok(())
}

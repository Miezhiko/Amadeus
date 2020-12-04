use crate::{
  types::common::PubCreds,
  common::{ msg::channel_message
          , msg::direct_message
          , help::members::get_player },
  steins::cyber::flo::*
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use flo_grpc::controller::*;
use flo_grpc::player::*;
use flo_grpc::game::*;

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

/*
enum Race {
  RaceHuman = 0;
  RaceOrc = 1;
  RaceNightElf = 2;
  RaceUndead = 3;
  RaceRandom = 4;
}
*/

#[command]
#[owners_only]
async fn create_game_vs_amadeus(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;

  let player_slot_settings = SlotSettings {
    team: 1,
    color: 1,
    handicap: 100,
    status: 2,
    race: 3,
    ..Default::default()
  };

  let amadeus_slot_settings = SlotSettings {
    team: 2,
    color: 2,
    computer: 2,
    handicap: 100,
    status: 2,
    race: 4,
    ..Default::default()
  };

  let res = rpc
    .create_game_as_bot(CreateGameAsBotRequest {
      name: "TEST".to_string(),
      map: Some(get_map()?),
      node_id: 14, //russia
      slots: vec![
        CreateGameSlot {
          player_id: Some(317),
          settings: Some(player_slot_settings),
          ..Default::default()
        },
        CreateGameSlot {
          player_id: None,
          settings: Some(amadeus_slot_settings),
          ..Default::default()
        }
      ],
      ..Default::default()
    })
    .await?;

  let id = res.into_inner().game.unwrap().id;

  let game_start = rpc
    .start_game_as_bot(StartGameAsBotRequest { game_id: id })
    .await?
    .into_inner();

  channel_message(ctx, msg, &format!("Game {} started: {:?}", id, game_start)).await;

  Ok(())
}

#[command]
//#[min_args(1)]
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

  let id = res.into_inner().game.unwrap().id;

  let game_start = rpc
    .start_game_as_bot(StartGameAsBotRequest { game_id: id })
    .await?
    .into_inner();

  channel_message(ctx, msg, &format!("Game {} started: {:?}", id, game_start)).await;

  Ok(())
}

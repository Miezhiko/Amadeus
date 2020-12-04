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
#[owners_only]
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

#[command]
async fn host_vs_amadeus(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;

  let user_secret_res = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: msg.author.name.clone(),
      source_id: msg.author.id.0.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let race_num: i32 =
    if let Ok(race_vs) = args.single_quoted::<String>() {
      get_race_num(&race_vs)
    } else {
      4
    };

  direct_message(ctx, msg, &format!("your token: {}", user_secret_res.token)).await;
  if let Some(p) = user_secret_res.player {
    let player_slot_settings = SlotSettings {
      team: 1,
      color: 1,
      handicap: 100,
      status: 2,
      race: race_num,
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
            player_id: Some(p.id),
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
  } else {
    channel_message(ctx, msg, "Failed to get player").await;
  }

  Ok(())
}

#[command]
#[min_args(1)]
async fn host_vs(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;

  let user_secret_res1 = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: msg.author.name.clone(),
      source_id: msg.author.id.0.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let user_secret_res2 = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: meme.user.name.clone(),
      source_id: meme.user.id.0.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let race_num1: i32 =
    if let Ok(race_vs) = args.single_quoted::<String>() {
      get_race_num(&race_vs)
    } else {
      4
    };
  let race_num2: i32 =
    if let Ok(race_vs) = args.single_quoted::<String>() {
      get_race_num(&race_vs)
    } else {
      4
    };

  direct_message(ctx, msg, &format!("your token: {}\nopponent token: {}"
                           , user_secret_res1.token, user_secret_res2.token)).await;

  if let Some(p1) = user_secret_res1.player {
    if let Some(p2) = user_secret_res2.player {

      let player1_slot_settings = SlotSettings {
        team: 1,
        color: 1,
        handicap: 100,
        status: 2,
        race: race_num1,
        ..Default::default()
      };

      let player2_slot_settings = SlotSettings {
        team: 2,
        color: 2,
        handicap: 100,
        status: 2,
        race: race_num2,
        ..Default::default()
      };

      let res = rpc
        .create_game_as_bot(CreateGameAsBotRequest {
          name: "TEST".to_string(),
          map: Some(get_map()?),
          node_id: 14, //russia
          slots: vec![
            CreateGameSlot {
              player_id: Some(p1.id),
              settings: Some(player1_slot_settings),
              ..Default::default()
            },
            CreateGameSlot {
              player_id: Some(p2.id),
              settings: Some(player2_slot_settings),
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
    }
  }

  Ok(())
}

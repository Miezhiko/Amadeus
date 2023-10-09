use crate::{
  common::{ msg::channel_message
          , msg::direct_message
          , help::members::get_player },
  steins::warcraft::flo::*
};

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateEmbed, CreateEmbedFooter },
  model::channel::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

use chrono::NaiveDateTime;

use flo_grpc::controller::*;
use flo_grpc::player::*;
use flo_grpc::game::*;

#[command]
#[aliases(nodes)]
async fn flo_nodes(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  let mut rpc = get_grpc_client().await;
  let nodes_reply = rpc.list_nodes(()).await?;
  let nodes = nodes_reply.into_inner().nodes;
  let n_strs = nodes.iter()
                    .map(|n| format!("**{}** {} [{}] *{}*"
                                    , n.name
                                    , n.ip_addr
                                    , n.country_id
                                    , n.location))
                    .collect::<Vec<String>>();
  let footer = format!("Requested by {}", msg.author.name);
  if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(CreateEmbed::new()
    .description(n_strs.join("\n"))
    .footer(CreateEmbedFooter::new(footer))
  )).await {
    error!("Failed to post nodes {why}");
  }
  Ok(())
}

#[command]
async fn flo_bans(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  let mut rpc = get_grpc_client().await;
  let bans_reply = rpc.list_player_bans(ListPlayerBansRequest {
    ..Default::default()
  }).await?;
  let bans = bans_reply.into_inner().player_bans;
  let mut n_strs = vec![];
  for ban in bans {
    if let Some(p) = ban.player {
      let mut expires = String::new();
      if let Some(e) = ban.ban_expires_at {
        let dt = NaiveDateTime::from_timestamp(e.seconds, e.nanos as u32);
        expires = dt.format("%Y-%m-%d %H:%M:%S").to_string();
      }
      n_strs.push(
        format!("**{}** {} [{}]",
          p.name, ban.ban_type, expires)
      );
    }
  }
  let footer = format!("Requested by {}", msg.author.name);
  if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(CreateEmbed::new()
    .description(n_strs.join("\n"))
    .footer(CreateEmbedFooter::new(footer))
  )).await {
    error!("Failed to post bans {why}");
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[owners_only]
async fn register_player(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
  let mut rpc = get_grpc_client().await;
  let res = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: meme.user.name,
      source_id: meme.user.id.to_string(),
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
async fn register_me(ctx: &Context, msg: &Message) -> CommandResult {
  let mut rpc = get_grpc_client().await;
  let res = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: msg.author.name.clone(),
      source_id: msg.author.id.to_string(),
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
  let mut rpc = get_grpc_client().await;
  let user_secret_res = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: msg.author.name.clone(),
      source_id: msg.author.id.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let map: Map =
    if let Ok(wanted_map) = args.single_quoted::<String>() {
      if let Ok(mapx) = get_map_by_name(&wanted_map).await {
        mapx
      } else {
        get_map()?
      }
    } else {
      get_map()?
    };

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

  if let Some(p) = user_secret_res.player {
    let player_slot_settings = SlotSettings {
      team: 0,
      color: 1,
      handicap: 100,
      status: 2,
      race: race_num1,
      ..Default::default()
    };

    let amadeus_slot_settings = SlotSettings {
      team: 1,
      color: 2,
      computer: 2,
      handicap: 100,
      status: 2,
      race: race_num2,
    };

    let res = rpc
      .create_game_as_bot(CreateGameAsBotRequest {
        name: msg.author.name.clone(),
        map: Some(map),
        node_id: 14, //russia
        slots: vec![
          CreateGameSlot {
            player_id: Some(p.id),
            settings: Some(player_slot_settings),
          },
          CreateGameSlot {
            player_id: None,
            settings: Some(amadeus_slot_settings),
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

    channel_message(ctx, msg, &format!("Game {id} started: {:?}", game_start)).await;
  } else {
    channel_message(ctx, msg, "Failed to get player").await;
  }

  Ok(())
}

#[command]
#[min_args(1)]
async fn host_vs(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let meme = get_player(&args.single_quoted::<String>()?, ctx, msg).await?;
  let mut rpc = get_grpc_client().await;

  let user_secret_res1 = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: msg.author.name.clone(),
      source_id: msg.author.id.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let user_secret_res2 = rpc
    .update_and_get_player(UpdateAndGetPlayerRequest {
      source: PlayerSource::Api as i32,
      name: meme.user.name.clone(),
      source_id: meme.user.id.to_string(),
      ..Default::default()
    }).await?.into_inner();

  let map: Map =
    if let Ok(wanted_map) = args.single_quoted::<String>() {
      if let Ok(mapx) = get_map_by_name(&wanted_map).await {
        mapx
      } else {
        get_map()?
      }
    } else {
      get_map()?
    };

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

  if let Some(p1) = user_secret_res1.player {
    if let Some(p2) = user_secret_res2.player {

      let player1_slot_settings = SlotSettings {
        team: 0,
        color: 1,
        handicap: 100,
        status: 2,
        race: race_num1,
        ..Default::default()
      };

      let player2_slot_settings = SlotSettings {
        team: 1,
        color: 2,
        handicap: 100,
        status: 2,
        race: race_num2,
        ..Default::default()
      };

      let res = rpc
        .create_game_as_bot(CreateGameAsBotRequest {
          name: msg.author.name.clone(),
          map: Some(map),
          node_id: 14, //russia
          slots: vec![
            CreateGameSlot {
              player_id: Some(p1.id),
              settings: Some(player1_slot_settings),
            },
            CreateGameSlot {
              player_id: Some(p2.id),
              settings: Some(player2_slot_settings),
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

      channel_message(ctx, msg, &format!("Game {id} started: {:?}", game_start)).await;
    }
  }

  Ok(())
}

use crate::{
  types::common::PubCreds,
  common::{
    msg::channel_message,
    sled
  }
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{
    Args, CommandResult,
    macros::command
  }
};

pub use flo_grpc::controller::flo_controller_client::FloControllerClient;
use flo_grpc::Channel;
use flo_grpc::controller::*;
use flo_grpc::game::*;


use tonic::Request;

const MAP: &str = r#"maps\frozenthrone\(4)twistedmeadows.w3x"#;

async fn get_grpc_client(flo_secret: String) -> FloControllerClient<Channel> {
  let channel = Channel::from_static("tcp://service.w3flo.com:3549")
    .connect()
    .await
    .unwrap();
  FloControllerClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
    req
      .metadata_mut()
      .insert("x-flo-secret", flo_secret.parse().unwrap());
    Ok(req)
  })
}

/*
fn get_map() -> eyre::Result<Map> {
  let storage = flo_w3storage::W3Storage::from_env()?;
  let (map, checksum) = flo_w3map::W3Map::open_storage_with_checksum(&storage, MAP)?;
  let map = Map {
    sha1: checksum.sha1.to_vec(),
    checksum: u32::from_le_bytes([0xED, 0xB9, 0xC9, 0x08]),
    name: "FLO_CLI".to_string(),
    description: map.description().to_string(),
    author: map.author().to_string(),
    path: MAP.to_string(),
    width: map.dimension().0,
    height: map.dimension().1,
    players: map
      .get_players()
      .into_iter()
      .map(|v| MapPlayer {
        name: v.name.to_string(),
        r#type: v.r#type,
        race: v.race,
        flags: v.flags,
      })
      .collect(),
    forces: map
      .get_forces()
      .into_iter()
      .map(|v| MapForce {
        name: v.name.to_string(),
        flags: v.flags,
        player_set: v.player_set,
      })
      .collect(),
  };
  Ok(map)
}
*/

#[command]
async fn flo_nodes(ctx: &Context, msg: &Message) -> CommandResult {
  let flo_secret = {
    let data = ctx.data.read().await;
    data.get::<PubCreds>().unwrap().get("flo").unwrap().as_str().to_string()
  };
  let mut rpc = get_grpc_client(flo_secret).await;
  let nodes = rpc.list_nodes({}).await;
  channel_message(ctx, msg, &format!("{:?}", nodes)).await;
  Ok(())
}

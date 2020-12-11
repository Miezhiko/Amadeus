use flo_grpc::Channel;
use flo_grpc::controller::*;
use flo_grpc::game::*;

use async_std::fs;
use serde_json::Value;

pub fn get_race_num(race_x: &str) -> i32 {
  let race_vs_lower = race_x.to_lowercase();
  if race_vs_lower.starts_with('h') {
    0
  } else if race_vs_lower.starts_with('o') {
    1
  } else if race_vs_lower.starts_with('n')
          || race_vs_lower.starts_with('e') {
    2
  } else if race_vs_lower.starts_with('u') {
    3
  } else {
    4
  }
}

pub async fn get_grpc_client(flo_secret: String) -> flo_controller_client::FloControllerClient<Channel> {
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

// that's very default map when map name is not provided
pub fn get_map() -> eyre::Result<Map> {
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

pub async fn get_map_by_name(name: &str) -> eyre::Result<Map> {
  let maps_json = fs::read_to_string("maps/maps.json").await?;
  let json: Value = serde_json::from_str(&maps_json)?;
  let maps = json.as_array().unwrap();
  let mut maybe_maps = vec![];
  for map in maps.iter() {
    if let Some(map_name) = map.pointer("/name") {
      let mapn = map_name.as_str().unwrap().to_string();
      if mapn.contains(name) {
        maybe_maps.push(map);
      }
    }
  }

  if maybe_maps.is_empty() {
    return Err( eyre!("Can't find this map") );
  }

  let mut picked_map: &Value = maybe_maps.first().unwrap();
  // using last w3c map if possible intead of other variants
  for map in maybe_maps {
    if let Some(map_path) = map.pointer("/path") {
      let mapp = map_path.as_str().unwrap().to_string();
      if mapp.contains("W3Champions\\v5") {
        picked_map = map;
        break;
      }
    }
  }

  let path = picked_map.pointer("/path").unwrap().as_str().unwrap().to_string();
  let sha1 = picked_map.pointer("/sha1").unwrap().as_str().unwrap().to_string();
  let checksum = picked_map.pointer("/checksum").unwrap().as_u64().unwrap();
  let real_name = picked_map.pointer("/name").unwrap().as_str().unwrap().to_string();
  let author = picked_map.pointer("/author").unwrap().as_str().unwrap().to_string();
  let description = picked_map.pointer("/description").unwrap().as_str().unwrap().to_string();
  let width = picked_map.pointer("/width").unwrap().as_u64().unwrap();
  let height = picked_map.pointer("/height").unwrap().as_u64().unwrap();

  let mut payers = vec![];
  if let Some(players_j) = picked_map.pointer("/players") {
    let players_a = players_j.as_array().unwrap();
    for palyer_j in players_a {
      let pname = palyer_j.pointer("/name").unwrap().as_str().unwrap().to_string();
      let ptype = palyer_j.pointer("/type").unwrap().as_u64().unwrap();
      let pflags = palyer_j.pointer("/flags").unwrap().as_u64().unwrap();
      let map_player = MapPlayer {
        name: pname,
        r#type: ptype as u32,
        flags: pflags as u32,
        ..Default::default()
      };
      payers.push(map_player);
    }
  }

  let mut forces = vec![];
  if let Some(forces_j) = picked_map.pointer("/forces") {
    let forces_a = forces_j.as_array().unwrap();
    for force_j in forces_a {
      let fname = force_j.pointer("/name").unwrap().as_str().unwrap().to_string();
      let fplayer_set = force_j.pointer("/player_set").unwrap().as_u64().unwrap();
      let fflags = force_j.pointer("/flags").unwrap().as_u64().unwrap();
      let map_force = MapForce {
        name: fname,
        flags: fflags as u32,
        player_set: fplayer_set as u32,
        ..Default::default()
      };
      forces.push(map_force);
    }
  }

  let map = Map {
    sha1: hex::decode(sha1)?,
    checksum: checksum as u32,
    name: real_name,
    description: description,
    author: author,
    path: path,
    width: width as u32,
    height: height as u32,
    players: payers,
    forces: forces
  };
  Ok(map)
}

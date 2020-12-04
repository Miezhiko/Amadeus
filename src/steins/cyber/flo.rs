use flo_grpc::Channel;
use flo_grpc::controller::*;
use flo_grpc::game::*;

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

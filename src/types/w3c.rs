use std::collections::HashMap;

pub_struct!(Stats {
  race: u32,
  gateWay: u32,
  id: String,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64,
});

pub_struct!(WinLosses {
  race: u32,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64,
});

pub_struct!(WinLossesOnMap {
  map: String,
  winLosses: Vec<WinLosses>,
});

pub_struct!(RaceWinsOnMap {
  race: u32,
  winLossesOnMap: Vec<WinLossesOnMap>,
});

pub_struct!(Stats2 {
  id: String,
  raceWinsOnMap: Vec<RaceWinsOnMap>,
  raceWinsOnMapByPatch: HashMap<String, Vec<RaceWinsOnMap>>,
  battleTag: String,
  season: u32,
});

pub_struct!(PlayerId {
  name: String,
  battleTag: String,
});

pub_struct!(PadPlayer {
  playerIds: Vec<PlayerId>,
  name: String,
  id: String,
  mmr: u32,
  gateWay: u32,
  gameMode: u32,
  season: u32,
  wins: u32,
  losses: u32,
  games: u32,
  winrate: f64,
});

pub_struct!(RankingPointsProgress {
  rankingPoints: i32,
  mmr: i32,
});

pub_struct!(Search {
  gateway: u32,
  id: String,
  league: u32,
  rankNumber: u32,
  rankingPoints: u32,
  playerId: String,
  player: PadPlayer,
  gameMode: u32,
  season: u32,
});

pub_struct!(GMStats {
  race: Option<u32>,
  division: u32,
  gameMode: u32,
  games: u32,
  gateWay: u32,
  id: String,
  leagueId: u32,
  leagueOrder: u32,
  losses: u32,
  mmr: u32,
  playerIds: Vec<PlayerId>,
  rank: u32,
  rankingPoints: u32,
  rankingPointsProgress: RankingPointsProgress,
  season: u32,
  winrate: f64,
  wins: u32,
});

pub_struct!(TeamPlayer {
  race: u32,
  oldMmr: u32,
  currentMmr: u32,
  battleTag: String,
  name: String,
  mmrGain: i32,
  won: bool,
});

pub_struct!(Team {
  players: Vec<TeamPlayer>,
  won: bool,
});

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct PlayerServerInfo {
  pub averagePing: u32,
  pub battleTag: String,
  pub currentPing: u32
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct ServerInfo {
  pub countryCode: Option<String>,
  pub location: Option<String>,
  pub name: Option<String>,
  pub nodeId: Option<u32>,
  pub playerServerInfos: Vec<PlayerServerInfo>,
  pub provider: String
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Match {
  pub map: String,
  pub id: String,
  #[serde(rename = "original-ongoing-match-id")]
  pub match_id: String,
  pub durationInSeconds: u32,
  pub startTime: String,
  pub endTime: String,
  pub gameMode: u32,
  pub serverInfo: ServerInfo,
  pub teams: Vec<Team>,
  pub gateWay: u32,
  pub season: u32
}

pub_struct!(Going {
  matches: Vec<Match>,
  count: u32,
});

pub_struct!(UnitScore {
  unitsProduced: u32,
  unitsKilled: u32,
  largestArmy: u32,
});

pub_struct!(Hero {
  icon: String,
  level: u32,
});

pub_struct!(HeroScore {
  heroesKilled: u32,
  itemsObtained: u32,
  mercsHired: u32,
  expGained: u32,
});

pub_struct!(ResourceScore {
  goldCollected: u32,
  lumberCollected: u32,
  goldUpkeepLost: u32,
});

pub_struct!(PlayerScore {
  battleTag: String,
  unitScore: UnitScore,
  heroes: Vec<Hero>,
  heroScore: HeroScore,
  resourceScore: ResourceScore,
});

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct MD {
  #[serde(rename = "match")]
  pub match_data: Match,
  pub playerScores: Vec<PlayerScore>
}

pub_struct!(Season {
  id: u32,
});

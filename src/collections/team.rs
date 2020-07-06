use crate::common::types::{ Player, player, streamer, streams, twitch };

//krolu_
//Schappster#2205
pub static DIVISION1: &'static [Player] = &
  [ player("Edek#1800", 298003054110965762)
  , player("RiplEy#21126", 137314790769950720)
  , streamer("ENA1337#2702", 138315254378659840,  twitch("ena1337"))
  , streamer("Soulful#2300", 147805125824610304,  twitch("soulful85"))
  , streamer("Muffy#2299", 196231768880185344,    twitch("muffy_supergosu"))
  , streamer("Trunks#2116", 297810232254398466,   twitch("trynkzzz"))
  , streamer("Маша#21994", 510368731378089984,    twitch("Qinon"))
  , streamer("Reyenir#2169", 287914378513088515,  streams("Cherry-Phoenix", "reyenir"))
  ];

pub static DIVISION2: &'static [Player] = &
  [ player("Alucard#1389", 196922236895100928)
  , player("Geriksmerik#2605", 444864857658097664)
  , player("RavenTT#2656", 277555411584942080)
  , player("ParadoCS#21514", 458674374044549140)
  , streamer("Ps1xoZ#21765", 404546426161332224,    twitch("ps1xoz_human"))
  , streamer("Лилуал#2400", 367722659590569994,     twitch("lilyal_"))
  , streamer("PETRUSHQUE#2357", 137402988539936769, twitch("PETRUSHQUE"))
  , streamer("Punisher#24744", 442391445341732864,  twitch("punisherl_l"))
  , streamer("SimbaPower#2413", 308475659775115265, streams("SimbaPower", "hemosimbapower"))
  , streamer("Rearden#21595", 622849650847907870,   twitch("amateurrearden"))
  , streamer("hhh#2991", 510732278025027596,        twitch("hotwussy"))
  ];

pub static INTERESTING: &'static [Player] = &
  [ streamer("Fingon#2350", 361930230375514112,         twitch("Skyrimoon"))
  , streamer("РозовыйПони#228941", 279559886931492865,  streams("JosephStalin", "sqktgw"))
  , streamer("ag3nt#21617", 293268448212156416,         streams("ag3nt", "ag3ntik"))
  ];

pub fn players() -> Vec<Player> {
  [DIVISION1, DIVISION2, INTERESTING].concat()
}

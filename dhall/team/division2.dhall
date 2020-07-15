let Streams : Type =
  { ggru: Optional Text
  , twitch: Optional Text
  }
let Player : Type =
  { battletag: Text
  , discord: Text
  , streams: Optional Streams
  }
let player = \(btag: Text) -> \(disc: Natural) ->
  { battletag = btag
  , discord   = disc
  , streams   = None Streams
  }
let streamer = \(btag: Text) -> \(disc: Natural) -> \(st: Streams) ->
  { battletag = btag
  , discord   = disc
  , streams   = Some st
  }

in  [ player   "Alucard#1389"     196922236895100928
    , player   "Geriksmerik#2605" 444864857658097664
    , player   "RavenTT#2656"     277555411584942080
    , player   "ParadoCS#21514"   458674374044549140
    , player   "Fopogg#2818"      318043743628427269
    , streamer "Ps1xoZ#21765"     404546426161332224 { ggru = None Text, twitch = Some "ps1xoz_human" }
    , streamer "Лилуал#2400"      367722659590569994 { ggru = None Text, twitch = Some "lilyal_" }
    , streamer "PETRUSHQUE#2357"  137402988539936769 { ggru = None Text, twitch = Some "PETRUSHQUE" }
    , streamer "Punisher#24744"   442391445341732864 { ggru = None Text, twitch = Some "punisherl_l" }
    , streamer "SimbaPower#2413"  308475659775115265 { ggru = Some "SimbaPower", twitch = Some "hemosimbapower" }
    , streamer "Rearden#21595"    622849650847907870 { ggru = None Text, twitch = Some "amateurrearden" }
    ]

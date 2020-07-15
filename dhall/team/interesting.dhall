let Streams : Type =
  { ggru: Optional Text
  , twitch: Optional Text
  }
let Player : Type =
  { battletag: Text
  , discord: Text
  , streams: Optional Streams
  }
let streamer = λ(btag: Text) -> λ(disc: Natural) -> λ(st: Streams) ->
  { battletag = btag
  , discord   = disc
  , streams   = Some st
  }

in [ streamer "Fingon#2350"        361930230375514112 { ggru = None Text, twitch = Some "Skyrimoon" }
   , streamer "РозовыйПони#228941" 279559886931492865 { ggru = Some "JosephStalin", twitch = Some "sqktgw" }
   , streamer "ag3nt#21617"        293268448212156416 { ggru = Some "ag3nt", twitch = Some "ag3ntik" }
   ]

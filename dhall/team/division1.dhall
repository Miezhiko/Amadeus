let Streams : Type =
  { ggru: Optional Text
  , twitch: Optional Text
  }
let Player : Type =
  { battletag: Text
  , discord: Text
  , streams: Optional Streams
  }
let player = λ(btag: Text) -> λ(disc: Natural) ->
  { battletag = btag
  , discord   = disc
  , streams   = None Streams
  }
let streamer = λ(btag: Text) -> λ(disc: Natural) -> λ(st: Streams) ->
  { battletag = btag
  , discord   = disc
  , streams   = Some st
  }

in  [ player   "Edek#1800"      298003054110965762
    , player   "RiplEy#21126"   137314790769950720
    , player   "GadjiPro#2558"  331504164204969984
    , player   "SasoRii#21896"  714711512282824755
    , streamer "ENA1337#2702"   138315254378659840 { ggru = None Text, twitch = Some "ena1337" }
    , streamer "Soulful#2300"   147805125824610304 { ggru = None Text, twitch = Some "soulful85" }
    , streamer "Muffy#2299"     196231768880185344 { ggru = None Text, twitch = Some "muffy_supergosu" }
    , streamer "Trunks#2116"    297810232254398466 { ggru = None Text, twitch = Some "trynkzzz" }
    , streamer "Маша#21994"     510368731378089984 { ggru = None Text, twitch = Some "Qinon" }
    , streamer "KroLo#11461"    313638735562407937 { ggru = None Text, twitch = Some "krolu_" }
    , streamer "Reyenir#2169"   287914378513088515 { ggru = Some "Cherry-Phoenix", twitch = Some "reyenir" }
    , streamer "hhh#2991"       510732278025027596 { ggru = None Text, twitch = Some "hotwussy" }
    ]

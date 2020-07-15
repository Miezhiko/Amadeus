let Streams : Type =
  { ggru: Optional Text, twitch: Optional Text }
let Player : Type =
  { battletag: Text, discord: Text, streams: Optional Streams }
let player = λ(btag: Text) -> λ(disc: Natural) ->
  { battletag = btag, discord   = disc, streams   = None Streams }
let twitch = λ(btag: Text) -> λ(disc: Natural) -> λ(tw: Text) ->
  { battletag = btag
  , discord   = disc
  , streams   = Some { ggru = None Text, twitch = Some tw } }
let twiggr = λ(btag: Text) -> λ(disc: Natural) -> λ(tw: Text) -> λ(gg: Text) ->
  { battletag = btag
  , discord   = disc
  , streams   = Some { ggru = Some gg, twitch = Some tw } }

in  [ player "Edek#1800"      298003054110965762
    , player "RiplEy#21126"   137314790769950720
    , player "GadjiPro#2558"  331504164204969984
    , player "SasoRii#21896"  714711512282824755
    , twitch "ENA1337#2702"   138315254378659840 "ena1337"
    , twitch "Soulful#2300"   147805125824610304 "soulful85"
    , twitch "Muffy#2299"     196231768880185344 "muffy_supergosu"
    , twitch "Trunks#2116"    297810232254398466 "trynkzzz"
    , twitch "Маша#21994"     510368731378089984 "Qinon"
    , twitch "KroLo#11461"    313638735562407937 "krolu_"
    , twitch "hhh#2991"       510732278025027596 "hotwussy"
    , twiggr "Reyenir#2169"   287914378513088515 "reyenir" "Cherry-Phoenix"
    ]

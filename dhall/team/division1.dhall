let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ p "Edek#1800"         298003054110965762
  , p "RiplEy#21126"      137314790769950720
  , p "GadjiPro#2558"     331504164204969984
  , p "SasoRii#21896"     714711512282824755
  , p "sockpuppet#21397"  204753768523235329
  , t "LosThopeLFT#2202"  313638735562407937 "krolu_"
  , t "ENA1337#2702"      138315254378659840 "ena1337"
  , t "Soulful#2300"      147805125824610304 "soulful85"
  , t "Muffy#2299"        196231768880185344 "muffy_supergosu"
  , t "Trunks#2116"       297810232254398466 "trynkzzz"
  , t "Маша#21994"        510368731378089984 "Qinon"
  , t "KroLo#11461"       313638735562407937 "krolu_"
  , t "hhh#2991"          510732278025027596 "hotwussy"
  , g "Reyenir#2169"      287914378513088515 "reyenir" "Cherry-Phoenix"
  , g "ag3nt#21617"       293268448212156416 "ag3ntik" "ag3nt"
  ]

in playersList

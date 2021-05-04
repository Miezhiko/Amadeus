let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let b = ./functions/b.dhall

let playersList : List Playerx =
  [ t "sockpuppet#21397"  204753768523235329 "whitemale18"
  , t "lmao#21922"        204753768523235329 "whitemale18"
  , t "PriesT#12658"      180213339111161856 "PriesTwc3"
  , t "ENA1337#2702"      138315254378659840 "ena1337"
  , t "Muffy#2299"        196231768880185344 "muffy_supergosu"
  , t "Trunks#2116"       297810232254398466 "trynkzzz"
  , t "Маша#21994"        510368731378089984 "Qinon"
  , t "hhh#2991"          510732278025027596 "hotwussy"
  , t "GraVe#2254"        355795504367534090 "GraVesc2"
  , t "SMDVKF#2721"       632300213494611968 "siriustvzzz"
  , b "Reyenir#2169"      287914378513088515 "reyenir" "Cherry-Phoenix"
  , b "ag3nt#21617"       293268448212156416 "ag3ntik" "ag3nt"
  , b "Rici#2903"         404379597568999425 "riciii" "Rici"
  , t "Avernus#2393"      247774118798295041 "ghost_17"
  , p "Edek#1800"         298003054110965762
  , p "RiplEy#21126"      137314790769950720
  , p "GadjiPro#2558"     331504164204969984
  , p "SasoRii#21896"     714711512282824755
  ]

in playersList

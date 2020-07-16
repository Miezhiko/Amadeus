let Playerx : Type = ./types/player_type.dhall

let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ t "Fingon#2350"        361930230375514112 "Skyrimoon"
  , g "РозовыйПони#228941" 279559886931492865 "sqktgw" "JosephStalin"
  , g "ag3nt#21617"        293268448212156416 "ag3ntik" "ag3nt"
  ]

in playersList

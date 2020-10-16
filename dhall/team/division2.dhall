let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ p "RavenTT#2656"        277555411584942080
  , p "Fopogg#2818"         318043743628427269
  , p "sonder#21813"        470291730328190985 -- rav3n
  , t "Rearden#21595"       622849650847907870 "amateurrearden"
  , g "Лилуал#2400"         367722659590569994 "lilyal_" "lilyal"
  ]

in playersList

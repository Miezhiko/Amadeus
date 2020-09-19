let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ p "RavenTT#2656"      277555411584942080
  , p "Fopogg#2818"       318043743628427269
  , p "Owi#21931"         201796316244869120
  , p "sonder#21813"      470291730328190985 -- rav3n
  , t "ParadoCS#21514"    458674374044549140 "paradocs12"
  , t "PETRUSHQUE#2357"   137402988539936769 "PETRUSHQUE"
  , t "Rearden#21595"     622849650847907870 "amateurrearden"
  , g "Лилуал#2400"       367722659590569994 "lilyal_" "lilyal"
  , g "SimbaPower#2413"   308475659775115265 "hemosimbapower" "SimbaPower"
  ]

in playersList

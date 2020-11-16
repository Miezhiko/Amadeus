let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ p "RavenTT#2656"        277555411584942080
  , p "Fopogg#2818"         318043743628427269
  , p "sonder#21813"        470291730328190985 -- rav3n
  , p "FATAL#21779"         386856633403310080
  , p "Farer#21146"         444864857658097664
  , p "EnragedOxy#2999"     576662560120766474
  , p "Singroso#2472"       677187656857419792
  , t "Ps1xoZ#21765"        404546426161332224 "ps1xoz_human"
  , t "Rays#2344"           212657754584645633 "follow_rays"
  , t "Soulful#2300"        147805125824610304 "soulful85"
  , t "Avernus#2393"        247774118798295041 "ghost_17"
  , g "Лилуал#2400"         367722659590569994 "lilyal_" "lilyal"
  ]

in playersList

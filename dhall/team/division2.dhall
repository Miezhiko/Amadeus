let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall
let b = ./functions/b.dhall

let playersList : List Playerx =
  [ t "sonder#21813"        470291730328190985 "reforgedrav3n"
  , t "Ps1xoZ#21765"        404546426161332224 "ps1xoz_human"
  , t "Rays#2344"           212657754584645633 "follow_rays"
  , t "Soulful#2300"        147805125824610304 "soulful85"
  , t "Avernus#2393"        247774118798295041 "ghost_17"
  , t "JustDoiTpLs#2907"    543121502711840799 "justdoitpls22"
  , g "Rici#2903"           404379597568999425 "Rici"
  , b "НееЖить#2559"        703241941114028032 "masterv_64" "Ghost_OL"
  , p "RavenTT#2656"        277555411584942080
  , p "Fopogg#2818"         318043743628427269
  , p "FATAL#21779"         386856633403310080
  , p "Farer#21146"         444864857658097664
  , p "EnragedOxy#2999"     576662560120766474
  , p "Singroso#2472"       677187656857419792
  , p "Pechenka#21766"      677187656857419792
  ]

in playersList

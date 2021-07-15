let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall
let b = ./functions/b.dhall

let playersList : List Playerx =
  [ t "sonder#21813"        470291730328190985 "reforgedrav3n"
  , t "Ps1xoZ#21765"        404546426161332224 "ps1xoz_human"
  , t "Rays#2344"           212657754584645633 "follow_rays"
  , t "JustDoiTpLs#2907"    543121502711840799 "justdoitpls22"
  , t "Soulful#2300"        147805125824610304 "soulful85"
  , t "RaVeN#212265"        707083717595627570 "raven71russ"
  , b "НееЖить#2559"        703241941114028032 "masterv_64" "Ghost_OL"
  , b "Лилуал#2400"         367722659590569994 "lilyal_" "lilyal"
  , p "RavenTT#2656"        277555411584942080
  , p "Singroso#2472"       677187656857419792
  , p "Pechenka#21766"      677187656857419792
  , p "Fopogg#2818"         318043743628427269
  , p "FATAL#21779"         386856633403310080
  , p "EnragedOxy#2999"     576662560120766474
  , p "WindWalker#21184"    720938816810254338
  ]

in playersList

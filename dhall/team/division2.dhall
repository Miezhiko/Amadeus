let Playerx : Type = ./types/player_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall

let playersList : List Playerx =
  [ p "Alucard#1389"     196922236895100928
  , p "Geriksmerik#2605" 444864857658097664
  , p "RavenTT#2656"     277555411584942080
  , p "ParadoCS#21514"   458674374044549140
  , p "Fopogg#2818"      318043743628427269
  , t "Ps1xoZ#21765"     404546426161332224 "ps1xoz_human"
  , t "Лилуал#2400"      367722659590569994 "lilyal_"
  , t "PETRUSHQUE#2357"  137402988539936769 "PETRUSHQUE"
  , t "Punisher#24744"   442391445341732864 "punisherl_l"
  , t "Rearden#21595"    622849650847907870 "amateurrearden"
  , g "SimbaPower#2413"  308475659775115265 "hemosimbapower" "SimbaPower"
  ]

in playersList

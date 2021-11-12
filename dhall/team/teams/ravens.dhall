let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let p = ./../functions/p.dhall
let t = ./../functions/t.dhall
let g = ./../functions/g.dhall
let b = ./../functions/b.dhall

let playersList : List Playerx =
  [ t "Miezhiko#2635"       510368731378089984 "Miezhiko"
  , t "hhh#2991"            510732278025027596 "hotwussy"
  , b "Reyenir#2169"        287914378513088515 "reyenir" "Cherry-Phoenix"
  , b "ag3nt#21617"         293268448212156416 "ag3ntik" "ag3nt"
  , b "Rici#2903"           404379597568999425 "riciii" "Rici"
  , t "Avernus#2393"        247774118798295041 "ghost_17"
  , t "sonder#21813"        470291730328190985 "reforgedrav3n"
  , t "JustDoiTpLs#2907"    543121502711840799 "justdoitpls22"
  , t "Soulful#2300"        147805125824610304 "soulful85"
  , t "YoungUndead#2289"    332258705456234497 "youngundeadd"
  , p "Edek#1800"           298003054110965762
  , p "RiplEy#21126"        137314790769950720
  , p "RavenTT#2656"        277555411584942080
  , p "Singroso#2472"       677187656857419792
  , p "Pechenka#21766"      677187656857419792
  ]

let discord : Discordx = { uid      = 872428638228844594
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = None Natural
                         , events   = None Natural
                         , players  = playersList }

in discord

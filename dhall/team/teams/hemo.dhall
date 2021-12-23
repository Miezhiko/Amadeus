let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let p = ./../functions/p.dhall
let t = ./../functions/t.dhall
let g = ./../functions/g.dhall
let b = ./../functions/b.dhall

let playersList : List Playerx =
  [ t "PriesT#12658"        180213339111161856 "PriesTwc3"
  , t "Muffy#2299"          196231768880185344 "muffy_supergosu"
  , { battletag = "Miezhiko#2635"
    , discord   = 510368731378089984
    , streams   = Some { ggru = None Text
                       , twitch = Some "Miezhiko"
                       }
    , other_acc = None "Маша#21485" }
  , t "hhh#2991"            510732278025027596 "hotwussy"
  , b "Reyenir#2169"        287914378513088515 "reyenir" "Cherry-Phoenix"
  , b "ag3nt#21617"         293268448212156416 "ag3ntik" "ag3nt"
  , b "Rici#2903"           404379597568999425 "riciii" "Rici"
  , t "Avernus#2393"        247774118798295041 "ghost_17"
  , t "Ps1xoZ#21765"        404546426161332224 "ps1xoz_human"
  , t "JustDoiTpLs#2907"    543121502711840799 "justdoitpls22"
  , t "Soulful#2300"        147805125824610304 "soulful85"
  , b "НееЖить#2559"        703241941114028032 "masterv_64" "Ghost_OL"
  , t "Fingon#2350"         361930230375514112 "Skyrimoon"
  , t "YoungUndead#2289"    332258705456234497 "youngundeadd"
  , p "FreeMasha#2272"      743770620671819776
  , p "Edek#1800"           298003054110965762
  , p "RiplEy#21126"        137314790769950720
  , p "RavenTT#2656"        277555411584942080
  , p "EnragedOxy#2999"     576662560120766474
  , p "OwerPidor#2503"      862690932931559464
  , { battletag = "WindWalker#21184"
    , discord   = 720938816810254338
    , streams   = None Streams
    , other_acc = None "War3Alex#2898" }
  ]

let discord : Discordx = { uid      = 611822838831251466
                         , games    = Some 721956117558853673
                         , games2   = Some 864417724445098004
                         , games4   = Some 864417767415349248
                         , streams  = Some 698766464420937768
                         , events   = Some 635912696675565608
                         , players  = playersList }

in discord

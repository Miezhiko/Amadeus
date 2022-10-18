let Playerx : Type = ./../types/player_type.dhall
let Streams : Type = ./../types/streams_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let p = ./../functions/p.dhall
let t = ./../functions/t.dhall
let g = ./../functions/g.dhall
let b = ./../functions/b.dhall

let playersList : List Playerx =
  [ { battletag = "Маша#21229"
    , discord   = 510368731378089984
    , streams   = Some { ggru   = Some "Miezhiko"
                       , twitch = Some "Miezhiko" }
    , alt_accounts = ["Miezhiko#2378"] }
  , b "ag3nt#21617"         293268448212156416 "ag3ntik" "ag3nt"
  , b "Rici#2903"           404379597568999425 "riciii" "Rici"
  , t "Avernus#2393"        247774118798295041 "ghost_17"
  , t "JustDoiTpLs#2907"    543121502711840799 "justdoitpls22"
  , t "Fingon#2350"         361930230375514112 "Skyrimoon"
  , t "YoungUndead#2289"    332258705456234497 "youngundeadd"
  , p "FreeMasha#2272"      743770620671819776
  , { battletag = "Smurf#24244"
    , discord   = 255345878917709824
    , streams   = None Streams
    , alt_accounts = ["Wayfarer#21400"] }
  , p "RiplEy#21126"        137314790769950720
  , p "RavenTT#2656"        277555411584942080
  ]

let discord : Discordx = { uid      = 611822838831251466
                         , games    = Some 721956117558853673
                         , games2   = Some 864417724445098004
                         , games4   = Some 864417767415349248
                         , streams  = Some 698766464420937768
                         , events   = Some 635912696675565608
                         , log      = Some 740913303278321704
                         , players  = playersList }

in discord

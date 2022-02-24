let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let p = ./../functions/p.dhall
let t = ./../functions/t.dhall
let g = ./../functions/g.dhall
let b = ./../functions/b.dhall

let playersList : List Playerx =
  [ t "Miezhiko#2635"       510368731378089984 "Miezhiko"
  , b "НееЖить#2559"        703241941114028032 "masterv_64" "Ghost_OL"
  , b "ag3nt#21617"         293268448212156416 "ag3ntik" "ag3nt"
  , t "Fingon#2350"         361930230375514112 "Skyrimoon"
  , p "FreeMasha#2272"      743770620671819776
  ]

let discord : Discordx = { uid      = 766693627639889980
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = None Natural
                         , events   = None Natural
                         , players  = playersList }

in discord

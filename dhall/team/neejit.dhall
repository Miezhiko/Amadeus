let Playerx : Type = ./types/player_type.dhall
let Discordx : Type = ./types/discord_type.dhall

let p = ./functions/p.dhall
let t = ./functions/t.dhall
let g = ./functions/g.dhall
let b = ./functions/b.dhall

let playersList : List Playerx =
  [ t "Miezhiko#2635"       510368731378089984 "Miezhiko"
  , b "НееЖить#2559"        703241941114028032 "masterv_64" "Ghost_OL"
  ]

let discord : Discordx = { uid      = 827134340277862400
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = None Natural
                         , events   = None Natural
                         , players  = playersList }

in discord

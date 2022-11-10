let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let t = ./../functions/t.dhall

let playersList : List Playerx =
  [ t "" 510368731378089984 "Miezhiko"
  , t "" 82981134597619712 "nopjmp"
  , t "" 267080914134237184 "ethannij"
  , t "" 85124478685675520  "xartin"
  ]

let discord : Discordx = { uid      = 249111029668249601
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = Some 913860369112506419
                         , events   = None Natural
                         , log      = Some 947749304339169310
                         , players  = playersList }

in discord

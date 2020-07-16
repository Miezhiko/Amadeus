let Streams : Type = ../types/streams_type.dhall
let Playerx : Type = ../types/player_type.dhall
in λ(btag: Text)
 → λ(disc: Natural) → { battletag = btag
                      , discord = disc
                      , streams = None Streams }

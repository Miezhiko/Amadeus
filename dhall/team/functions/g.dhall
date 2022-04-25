let Streams : Type = ../types/streams_type.dhall
let Playerx : Type = ../types/player_type.dhall
in λ(btag: Text)
 → λ(disc: Natural)
 → λ(gg: Text) →
  { battletag = btag
  , discord   = disc
  , streams   = Some { ggru = Some gg
                     , twitch = None Text
                     }
  , alt_accounts = [] : List Text
  }

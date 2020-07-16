let Streams : Type = ../types/streams_type.dhall
let Playerx : Type = ../types/player_type.dhall
in λ(btag: Text) 
 → λ(disc: Natural)
 → λ(tw: Text) →
  { battletag = btag
  , discord   = disc
  , streams   = Some { ggru = None Text
                     , twitch = Some tw
                     }
  }

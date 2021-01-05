let Streams : Type = ../types/streams_type.dhall
let Playerx : Type = ../types/player_type.dhall
in λ(btag: Text)
 → λ(disc: Natural)
 → λ(tw: Text)
 → λ(gg: Text) →
  { battletag = btag
  , discord   = disc
  , streams   = Some { ggru = Some gg
                     , twitch = Some tw
                     }
  }

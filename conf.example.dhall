{- Init config file
 * discord: discord token
 ...
-}

let SType = < HEmo
            | Storage
            | Safe
            | Unsafe >
let Server : Type =
  { id: Natural
  , name: Text
  , kind: SType }
let u = λ(id: Natural)
      → λ(name: Text) → { id = id, name = name, kind = SType.Unsafe }
let s = λ(id: Natural)
      → λ(name: Text) → { id = id, name = name, kind = SType.Safe }
let serversList : List Server =
  [ { id = 611822838831251466, name = "HEmo", kind = SType.HEmo }
  , { id = 740144638375231489, name = "Amadeus", kind = SType.Storage }
  , u 676119422418550815 "Зарянка"
  , s 728694826203349072 "Rusty Tools"
  ]
in { discord              = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
   , guild                = 611822838831251466
   , amadeus_guild        = 740144638375231489
   , servers              = serversList
   , twitch_client_id     = "AAAAAAAAAAAAAAAAAAAAAA"
   , twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
   , tenor_key            = "AAAAAAAAAA"
   }

let L = < English
        | Russian
        | Bilingual >
let Channel : Type =
  { id: Text
  , lang: L }
let ru = λ(id: Text) → { id = id, lang = L.Russian }
let en = λ(id: Text) → { id = id, lang = L.English }
let bi = λ(id: Text) → { id = id, lang = L.Bilingual }
let channels : List Channel =
  [ bi "main"
  , en "gym"
  , en "human"
  , en "nightelf"
  , en "orc"
  , en "undead"
  , bi "dating"
  , bi "warcraft"
  , bi "team-chat"
  , bi "division-1"
  , bi "division-2"
  , bi "🌈climatestrike"
  , bi "🇯🇵日本語"
  , ru "💬главный-зал💬"
  , ru "🏰warcraft-3🏰"
  , ru "штаб-оппозиции"
  ]
in channels

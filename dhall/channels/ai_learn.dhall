let L = < English
        | Russian
        | Bilingual >
let Channel : Type =
  { id: Natural
  , lang: L }
let ru = λ(id: Natural) → { id = id, lang = L.Russian }
let en = λ(id: Natural) → { id = id, lang = L.English }
let bi = λ(id: Natural) → { id = id, lang = L.Bilingual }
let channels : List Channel =
  [ bi 611822932897038341 -- "main"
  , bi 773830849124106250 -- "english"
  , bi 679723943535312903 -- "dating"
  , bi 611825511433240577 -- "warcraft"
  , bi 631379331515678720 -- "team-chat-en"
  , ru 841643803563655198 -- "team-chat-ru"
  , bi 695909020732620830 -- "division-2"
  , bi 666614389805416448 -- "🌈climatestrike"
  , bi 611824913829068800 -- "🇯🇵日本語"
  , ru 827151604053835807 -- nejit1
  , ru 827154134163390535 -- nejit2
  ]
in channels

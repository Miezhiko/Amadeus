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
  , en 773830849124106250 -- "english"
  , bi 679723943535312903 -- "dating"
  , bi 611825511433240577 -- "warcraft"
  , bi 631379331515678720 -- "team-chat"
  , bi 611824891666104320 -- "🚧random"
  , bi 755768364390940692 -- "💻computers"
  , bi 666614389805416448 -- "🌈climatestrike"
  , ru 676119422863278115 -- "💬главный-зал💬"
  , ru 676136749063209000 -- "🏰warcraft-3🏰"
  , ru 766697158245089310 -- "площадь-подземелья"
  , ru 766759137760903200 -- "warcraft3"
  , ru 766697312659177503 --"поныть-на-баланс"
  ]
in channels

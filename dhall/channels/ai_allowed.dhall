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
  , ru 827151604053835807 -- nejit1
  , ru 827154134163390535 -- nejit2
  , ru 766697158245089310 -- korchma 1
  , ru 802227624964522054 -- korchma 2
  , ru 766759137760903200 -- korchma 3
  , ru 766697312659177503 -- korchma 4
  , ru 767500986465648641 -- korchma 5
  ]
in channels

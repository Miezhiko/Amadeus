let L = < English
        | Russian
        | Bilingual >

let Channel : Type =
  { id: Natural
  , lang: L }

let en = λ(id: Natural) → { id = id, lang = L.English }
let ru = λ(id: Natural) → { id = id, lang = L.Russian }
let bi = λ(id: Natural) → { id = id, lang = L.Bilingual }

let channels : List Channel =
  [ bi 611822932897038341 -- "main"
  , bi 773830849124106250 -- "english"
  , ru 948097924649353287 -- "okremiy"
  , bi 679723943535312903 -- "dating"
  , bi 611825511433240577 -- "warcraft"
  , bi 666614389805416448 -- "🌈climatestrike"
  , bi 611824913829068800 -- "🇯🇵日本語"
  , bi 611824891666104320 -- "random"
  , bi 755768364390940692 -- "computers"
  , bi 1010570616719544330 -- "films"
  , en 1052777234454294569 -- learn something
  , en 249111029668249601 -- learn something else
  ]

in channels

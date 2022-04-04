let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let t = ./../functions/t.dhall

let playersList : List Playerx =
  [ t "" 510368731378089984 "Miezhiko"
  , t "" 82981134597619712 "nopjmp"
  , t "" 267080914134237184 "ethannij"
  , t "" 85124478685675520  "xartin"
  , t "" 313372908627951616 "upsilondiesbackwards"
  , t "" 597418436683104266 "allenchen632293"
  , t "" 666971070972166145 "maxxolox"
  , t "" 617111721080913975 "tommyluco"
  , t "" 652282204889546782 "thowilwin0"
  , t "" 452615881516646420 "blox_caster"
  , t "" 188198036525023232 "gannokvg"
  , t "" 162595998009524226 "y0rune"
  , t "" 89805412676681728 "rheaayase"
  , t "" 356224301248282624 "psilovybin_"
  , t "" 159855586039562240 "luke086"
  , t "" 285148358815776768 "starrfox"
  , t "" 790690784143605831 "lmunr"
  , t "" 147677951683461120 "hanro50"
  , t "" 530987638392487948 "mortada_killer0"
  , t "" 341088451644620800 "tuxlovesyou"
  , t "" 635520103701807104 "star4fun"
  , t "" 403340271825190933 "nihil8492"
  , t "" 274938691900669953 "zaechus"
  , t "" 295561288325791745 "sindibad363"
  , t "" 265995727409381377 "hersen"
  ]

let discord : Discordx = { uid      = 249111029668249601
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = Some 913860369112506419
                         , events   = None Natural
                         , log      = Some 947749304339169310
                         , players  = playersList }

in discord

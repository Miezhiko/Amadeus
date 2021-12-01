let Playerx : Type = ./../types/player_type.dhall
let Discordx : Type = ./../types/discord_type.dhall

let t = ./../functions/t.dhall

let playersList : List Playerx =
  [ t "" 510368731378089984 "Miezhiko"
  , t "" 82981134597619712 "nopjmp"
  , t "" 267080914134237184 "ethannij"
  , t "" 85124478685675520  "xartin"
  , t "" 725303688192720977 "mdddmmmm"
  , t "" 313372908627951616 "upsilondiesbackwards"
  , t "" 597418436683104266 "allenchen632293"
  , t "" 666971070972166145 "maxxolox"
  , t "" 485255323502772255 "oterminal"
  , t "" 617111721080913975 "tommyluco"
  , t "" 652282204889546782 "thowilwin0"
  , t "" 452615881516646420 "blox_caster"
  , t "" 188198036525023232 "gannokvg"
  , t "" 297477415171784715 "thehighestpeak"
  , t "" 127708549118689280 "dotinitrd"
  , t "" 162595998009524226 "y0rune"
  , t "" 89805412676681728 "rheaayase"
  , t "" 356224301248282624 "psilovybin_"
  , t "" 159855586039562240 "luke086"
  , t "" 601540783220195350 "endermen1094"
  , t "" 285148358815776768 "starrfox"
  , t "" 790690784143605831 "lmunr"
  , t "" 147677951683461120 "hanro50"
  , t "" 530987638392487948 "mortada_killer0"
  ]

let discord : Discordx = { uid      = 249111029668249601
                         , games    = None Natural
                         , games2   = None Natural
                         , games4   = None Natural
                         , streams  = Some 913860369112506419
                         , events   = None Natural
                         , players  = playersList }

in discord

<h1 align="center">
  Amadeus
  <br>
</h1>

<h4 align="center">Memory storage and artificial intelligence system.</h4>

<p align="center">
  <a href="#features">Features</a>
  •
  <a href="#cooking">Cooking</a>
  •
  <a href="#service">Service</a>
  •
  <a href="#notes">Notes</a>
</p>

[![Azure DevOps builds](https://img.shields.io/azure-devops/build/qeenon/0dcb863b-80ca-4465-a7f2-a8cb387865f9/1?color=blue&label=Azure)]()
[![GitHub](https://github.com/Qeenon/Amadeus/workflows/mawa/badge.svg?branch=mawa)](https://github.com/Qeenon/Amadeus/workflows/mawa)
[![CircleCI](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa.svg?style=shield)](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa)
[![Appveyor](https://ci.appveyor.com/api/projects/status/8cd1qi1aykujkyd2?svg=true)](https://ci.appveyor.com/project/Qeenon/amadeus)
[![GitHub contributors](https://img.shields.io/github/contributors/Qeenon/Amadeus.svg?style=flat)]()
[![GitHub last commit](https://img.shields.io/github/last-commit/Qeenon/Amadeus.svg?style=flat)]()
[![codecov](https://codecov.io/gh/Qeenon/Amadeus/branch/master/graph/badge.svg)](https://codecov.io/gh/Qeenon/Amadeus)
[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/Qeenon/Amadeus)
[![Discord](https://img.shields.io/discord/611822838831251466?label=Discord&color=pink)](https://discord.gg/GdzjVvD)
[![Twitter Follow](https://img.shields.io/twitter/follow/Qeenon.svg?style=social)](https://twitter.com/Qeenon)


## Features

<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

 - Too many small commands (use `~help`)
 - Chatty (`set activity 66` is default)
 - Transfer learning using chats context
 - Automatic translation with bert models
 - Rule-based grammatical error correction
 - [Deepspeach](https://github.com/mozilla/DeepSpeech) for voice[WIP]
 - Live games tracking on [w3champions](https://www.w3champions.com)
 - W3info news tracking using calendar
 - Replays parsing (click emoji to get report)
 - Points system on [Cannyls](https://github.com/frugalos/cannyls/wiki)
 - Almost everything async [tokio.rs](https://tokio.rs)
 - Various gifs commands using Tenor API
 - [Dhall](https://dhall-lang.org) and [rudano](https://github.com/pheki/rudano) config files
 - Using [Fluent](https://www.projectfluent.org/) for localization
 - [Sled](https://github.com/spacejam/sled) for editable info archive
 - Plays music streams! (`~join ~play`)
 - Multi-server streams notifications/trackers for twitch and goodgame.ru
 - Veto helper (for banning maps against some player) using W3C statistics
 - Versus command showing score for one player against another for x seasons
 - Bets on live games with `~bet` and emojis under Live tracking games
 - Warcraft 3 commands `~stats`, `~today` and more with info from wacraft3.info and W3C ladder
 - Warcraft 3 hostbot API integration (creating games) using flo services and tonic for RPC


## Cooking

 - to compile just use `cargo build --release` (on Windows you there could be problems with PyTorch, you can install it first and setup paths alike it's done in `misc/build.bat` file)
 - `cp conf.example.dhall conf.dhall` (initial constant options)
 - `cp conf.example.rs conf.rs` (those options may change in runtime)
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.dhall and fill `discord`, `tenor_key` and optionally `twitch` client data
 - conf.rs `twitch` value is OAuth access token, you can regenerate it with bot command `~twitch_token_update`

``` haskell
let SType = < Safe
            | Unsafe >
let Server : Type =
  { id: Natural, kind: SType }
let u = λ(id: Natural) → { id = id, kind = SType.Unsafe }
let s = λ(id: Natural) → { id = id, kind = SType.Safe }
let additional_servers : List Server =
  [ u 676119422418550815 -- "Unsafe Server"
  , s 728694826203349072 -- "Rusty Tools"
  ]
in { discord              = "AAAAAAAAA.AAA.AAAA-AAAAAAA"
   , app_id               = 000000000000000000
   , guild                = 611822838831251466
   , amadeus_guild        = 740144638375231489
   , servers              = additional_servers
   , twitch_client_id     = "AAAAAA"
   , twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAAAAA"
   , tenor_key            = "AAAA"
   , flo_secret           = "AAAAAAAAAAAAAAA"
   }
```

installation environment for typescript parsing backend (optional)
typescript replay parsing will be replaced in future.

```shell
npm install -D typescript
npm install -D ts-node
npm i @types/node
npm install w3gjs
```

to test typescript code you may run

```shell
node node_modules/ts-node/dist/bin.js ./js/w3g_parse.ts ./LastReplay.w3g > gg.out
```

Note: will lag at first run due pre-trained models downloading.
The models will be downloaded to the environment variable `RUSTBERT_CACHE` if it exists, otherwise to `~/.cache/.rustbert`

## Service

```shell
cp misc/Amadeus.service /etc/systemd/system/Amadeus.service
systemctl daemon-reload
systemctl enable Amadeus
systemctl start Amadeus
```

it's safe to rebuild and restart it

```shell
systemctl restart Amadeus
```

## Notes

 - Check TODO.md for planned work
 - You need libdeepspeech for voice analysis feature
 - Code open sourced just for my friend Sirius to be able to see it, please pay attention to license
 - Fingon is cute

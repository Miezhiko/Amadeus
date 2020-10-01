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
  •
  <a href="#license">License</a>
</p>

![Azure DevOps builds](https://img.shields.io/azure-devops/build/qeenon/0dcb863b-80ca-4465-a7f2-a8cb387865f9/1?color=blue&label=Azure)
[![Travis](https://travis-ci.org/Qeenon/Amadeus.svg?branch=master)](https://travis-ci.org/Qeenon/Amadeus)
[![GitHub](https://github.com/Qeenon/Amadeus/workflows/mawa/badge.svg?branch=mawa)](https://github.com/Qeenon/Amadeus/workflows/mawa)
[![CircleCI](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa.svg?style=shield)](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa)
[![Appveyor](https://ci.appveyor.com/api/projects/status/8cd1qi1aykujkyd2?svg=true)](https://ci.appveyor.com/project/Qeenon/amadeus)
[![License: ISC](https://img.shields.io/badge/License-ISC-teal.svg)](https://opensource.org/licenses/ISC)
[![GitHub contributors](https://img.shields.io/github/contributors/Qeenon/Amadeus.svg?style=flat)]()
[![GitHub last commit](https://img.shields.io/github/last-commit/Qeenon/Amadeus.svg?style=flat)]()
[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/Qeenon/Amadeus)
[![codecov](https://codecov.io/gh/Qeenon/Amadeus/branch/master/graph/badge.svg)](https://codecov.io/gh/Qeenon/Amadeus)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=small)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_small)
[![Discord](https://img.shields.io/discord/611822838831251466?label=Discord&color=pink)](https://discord.gg/GdzjVvD)
[![Twitter Follow](https://img.shields.io/twitter/follow/Qeenon.svg?style=social)](https://twitter.com/Qeenon) 


## Features

<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

 - many small commands (use `~help`)
 - chatty (`set activity 66` is default)
 - transfer learning using chats context
 - automatic translation with bert models
 - live games tracking on [w3champions](https://www.w3champions.com)
 - bets on live games with `~bet` command
 - warcraft 3 commands `~stats`, `~today` and more
 - w3info news tracking using calendar
 - replays parsing (click emoji to get report)
 - points system on [Cannyls](https://github.com/frugalos/cannyls/wiki)
 - async [tokio.rs](https://tokio.rs)
 - gifs commands
 - plays music streams!
 - [Dhall](https://dhall-lang.org) and [rudano](https://github.com/pheki/rudano) config files
 - stream notifications/trackers for twitch and goodgame.ru
 - using [Fluent](https://www.projectfluent.org/) for localization

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
  [ u 676119422418550815 -- "Зарянка"
  , s 728694826203349072 -- "Rusty Tools"
  ]
in { discord              = "AAAAAAAAA.AAA.AAAA-AAAAAAA"
   , guild                = 611822838831251466
   , amadeus_guild        = 740144638375231489
   , servers              = additional_servers
   , twitch_client_id     = "AAAAAA"
   , twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAAAAA"
   , tenor_key            = "AAAA"
   }
```

installation environment for typescript parsing backend (optional)
```shell
npm install -D typescript
npm install -D ts-node
npm i @types/node
npm install w3gjs
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

 - Work in progress
 - Check TODO.md

## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_large)

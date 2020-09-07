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

[![Build Status](https://travis-ci.org/Qeenon/Amadeus.svg?branch=master)](https://travis-ci.org/Qeenon/Amadeus)
[![mawa](https://github.com/Qeenon/Amadeus/workflows/mawa/badge.svg?branch=mawa)](https://github.com/Qeenon/Amadeus/workflows/mawa)
[![CircleCI](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa.svg?style=shield)](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa)
[![Appveyor](https://ci.appveyor.com/api/projects/status/8cd1qi1aykujkyd2?svg=true)](https://ci.appveyor.com/project/Qeenon/amadeus)
[![Codefresh build status]( https://g.codefresh.io/api/badges/pipeline/hemo/Amadeus%2Fmawa?type=cf-1)]( https://g.codefresh.io/public/accounts/hemo/pipelines/new/5f47c6dc6db0ba1ef73c93c8)
[![Percentage of issues still open](http://isitmaintained.com/badge/open/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Percentage of issues still open")
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Average time to resolve an issue")
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.44.1-teal)
[![License: ISC](https://img.shields.io/badge/License-ISC-teal.svg)](https://opensource.org/licenses/ISC)
[![Issues](https://img.shields.io/github/issues-raw/Qeenon/Amadeus.svg?maxAge=25000)](https://github.com/tterb/Hyde/issues)
[![GitHub contributors](https://img.shields.io/github/contributors/Qeenon/Amadeus.svg?style=flat)]()
[![GitHub last commit](https://img.shields.io/github/last-commit/Qeenon/Amadeus.svg?style=flat)]()
[![GitHub stars](https://img.shields.io/github/stars/Qeenon/Amadeus.svg?style=social&label=Stars&style=plastic)]()
[![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/Qeenon/Amadeus)
[![codecov](https://codecov.io/gh/Qeenon/Amadeus/branch/master/graph/badge.svg)](https://codecov.io/gh/Qeenon/Amadeus)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=small)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_small)
[![Discord](https://img.shields.io/discord/611822838831251466?label=Discord)](https://discord.gg/GdzjVvD)
[![Open Source Helpers](https://www.codetriage.com/qeenon/amadeus/badges/users.svg)](https://www.codetriage.com/qeenon/amadeus)
[![Twitter Follow](https://img.shields.io/twitter/follow/Qeenon.svg?style=social)](https://twitter.com/Qeenon) 


## Features

<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

 - chatty (ability to change activity level)
 - transfer learning using chats context
 - automatic translation with bert models
 - many small commands (use `~help`)
 - using [Dhall](https://dhall-lang.org) config files for things
 - live games tracking on [w3champions](https://www.w3champions.com)
 - some more warcraft 3 related stuff, like player stats and news
 - w3g replays parsing (early version)
 - points system on [Cannyls](https://github.com/frugalos/cannyls/wiki)
 - runs async, using [tokio.rs](https://tokio.rs)
 - many gifs commands for fun
 - plays music streams!
 - stream notifications/trackers for twitch and goodgame.ru

Using:

 - DistilBERT model finetuned on SQuAD (Stanford Question Answering Dataset)
 - MarianMT architecture and pre-trained models from the Opus-MT team from Language Technology at the University of Helsinki
 - Conversation model based on Microsoft's [DialoGPT](https://github.com/microsoft/DialoGPT)

The human evaluation results indicate that the response generated from DialoGPT is comparable to human response quality under a single-turn conversation Turing test. 

## Cooking

 - to compile just use `cargo build --release` (on Windows you there could be problems with PyTorch, you can install it first and setup paths alike it's done in `misc/build.bat` file)
 - `cp conf.example.dhall conf.dhall` (initial constant options)
 - `cp conf.example.json conf.json` (those options may change in runtime)
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.dhall and fill `tenor_key` and optionally `twitch` client data
 - conf.json `twitch` value is OAuth access token, you can regenerate it with bot command `~twitch_token_update`
 - create `UNBLOCK AMADEUS` role on server

``` haskell
{ discord              = "put discord token here"
, guild                = 0
, twitch_client_id     = "AAAAAAAAAAAAAAAAAAAAAAAA"
, twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAA"
, tenor_key            = "AAAAAAAAA"
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
The models will be downloaded to the environment variable RUSTBERT_CACHE if it exists, otherwise to ~/.cache/.rustbert

## Service

``` sh
cp misc/Amadeus.service /etc/systemd/system/Amadeus.service
systemctl daemon-reload
systemctl enable Amadeus
systemctl start Amadeus
```

note that you're fully safe to rebuild and restart it whenever you want

``` sh
systemctl restart Amadeus
```

## Notes

 - Only will work with server administrator permissions
 - Doesn't like other bots
 - Punish people for blocking her
 - Cruel


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_large)

Amadeus
=======

[![Build Status](https://travis-ci.org/Qeenon/Amadeus.svg?branch=master)](https://travis-ci.org/Qeenon/Amadeus)
[![CircleCI](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa.svg?style=shield)](https://circleci.com/gh/Qeenon/Amadeus/tree/mawa)
[![Appveyor](https://ci.appveyor.com/api/projects/status/8cd1qi1aykujkyd2?svg=true)](https://ci.appveyor.com/project/Qeenon/amadeus)
[![Percentage of issues still open](http://isitmaintained.com/badge/open/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Percentage of issues still open")
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Average time to resolve an issue")
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.44.1-teal)
[![License: ISC](https://img.shields.io/badge/License-ISC-teal.svg)](https://opensource.org/licenses/ISC)
![Discord](https://img.shields.io/discord/611822838831251466?label=Discord)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=small)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_small) [![Twitter Follow](https://img.shields.io/twitter/follow/Qeenon.svg?style=social)](https://twitter.com/Qeenon) 

Memory storage and Artificial Intelligence system.\
![mawa](https://github.com/Qeenon/Amadeus/workflows/mawa/badge.svg?branch=mawa) [![Issues](https://img.shields.io/github/issues-raw/Qeenon/Amadeus.svg?maxAge=25000)](https://github.com/tterb/Hyde/issues) [![GitHub contributors](https://img.shields.io/github/contributors/Qeenon/Amadeus.svg?style=flat)]() [![GitHub last commit](https://img.shields.io/github/last-commit/Qeenon/Amadeus.svg?style=flat)]() [![GitHub stars](https://img.shields.io/github/stars/Qeenon/Amadeus.svg?style=social&label=Stars&style=plastic)]() [![Gitpod ready-to-code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/Qeenon/Amadeus)\
Built with Rust and Serenity Framework.   [![Open Source Helpers](https://www.codetriage.com/qeenon/amadeus/badges/users.svg)](https://www.codetriage.com/qeenon/amadeus) [![codecov](https://codecov.io/gh/Qeenon/Amadeus/branch/master/graph/badge.svg)](https://codecov.io/gh/Qeenon/Amadeus)


Features
--------

<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

 - chatty (ability to change activity level)
 - transfer learning using chats context
 - automatic translation with bert models
 - many small commands (use `~help`)
 - using `Dhall` config files for things
 - live games tracking on w3champions
 - some more warcraft 3 related stuff, like player stats and news
 - points system on cannyls! https://github.com/frugalos/cannyls/wiki
 - runs async, using tokio https://tokio.rs
 - easily replaces Streamcord
 - many gifs commands for fun
 - plays music streams!

Using:

 - DistilBERT model finetuned on SQuAD (Stanford Question Answering Dataset)
 - MarianMT architecture and pre-trained models from the Opus-MT team from Language Technology at the University of Helsinki
 - Conversation model based on Microsoft's DialoGPT https://github.com/microsoft/DialoGPT

The human evaluation results indicate that the response generated from DialoGPT is comparable to human response quality under a single-turn conversation Turing test. 

Preparing
---------

 - to compile just use `cargo build --release`
 - `cp conf.example.dhall conf.dhall` (initial constant options)
 - `cp conf.example.json conf.json` (those options may change in runtime)
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.ini and fill `token` and optionally `[Twitch]` section
 - create `UNBLOCK AMADEUS` role on server

``` haskell
{ discord              = "put discord token here"
, guild                = 0
, twitch_oauth         = "Bearer AAAAAAAAAAAAAAAAAAAA"
, twitch_client_id     = "AAAAAAAAAAAAAAAAAAAAAAAA"
, twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAA"
, tenor_key            = "AAAAAAAAA"
}
```

Note: will lag at first run due pre-trained models downloading.
The models will be downloaded to the environment variable RUSTBERT_CACHE if it exists, otherwise to ~/.cache/.rustbert

Start as service
----------------

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

Note
====

 - Only will work with server administrator permissions
 - Doesn't like other bots
 - Punish people for blocking her
 - Cruel


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_large)

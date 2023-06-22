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
  <a href="#development">Development</a>
  •
  <a href="#service">Service</a>
  •
  <a href="#notes">Notes</a>
</p>

<h2 align="center">Looking for a drinks and hosting.</h2>

[![Build Status](https://dev.azure.com/miezhiko/Amadeus/_apis/build/status/Miezhiko.Amadeus?branchName=mawa)](https://dev.azure.com/miezhiko/Amadeus/_build/latest?definitionId=1&branchName=mawa)
[![GitHub](https://github.com/Miezhiko/Amadeus/workflows/mawa/badge.svg?branch=mawa)](https://github.com/Miezhiko/Amadeus/actions/workflows/mawa.yml)
[![Appveyor](https://ci.appveyor.com/api/projects/status/8cd1qi1aykujkyd2?svg=true)](https://ci.appveyor.com/project/Miezhiko/amadeus)
[![GitHub contributors](https://img.shields.io/github/contributors/Miezhiko/Amadeus.svg?style=flat)]()
[![GitHub last commit](https://img.shields.io/github/last-commit/Miezhiko/Amadeus.svg?style=flat)]()
[![codecov](https://codecov.io/gh/Miezhiko/Amadeus/branch/master/graph/badge.svg)](https://codecov.io/gh/Miezhiko/Amadeus)

[![Discord](https://img.shields.io/discord/611822838831251466?label=Discord&color=pink)](https://discord.gg/GdzjVvD)
[![Twitter Follow](https://img.shields.io/twitter/follow/Miezhiko.svg?style=social)](https://twitter.com/Miezhiko)


## Features

<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

 - Too many small [commands](https://www.youtube.com/watch?v=vVacOaFbrdE)
 - Slash commands support (type `/`)
 - [Chatty](https://www.youtube.com/watch?v=J_8cnSvHLLc) (`set activity 66` is default)
 - Automatic translation with bert models
 - Live games tracking on [w3champions](https://www.w3champions.com)
 - W3info news tracking using calendar
 - Replays parsing (click emoji to get report)
 - Points system on [Cannyls](https://github.com/frugalos/cannyls/wiki)
 - Emoji roles system on [Cannyls](https://github.com/frugalos/cannyls/wiki)
 - Almost everything async [tokio.rs](https://tokio.rs)
 - Various gifs commands using Tenor API
 - Using [Fluent](https://www.projectfluent.org/) for localization
 - [Sled](https://github.com/spacejam/sled) for editable info archive
 - Plays music streams using [Songbird](https://github.com/serenity-rs/songbird)! (`~join ~play`)
 - NLP with context (models: [DialoGPT](https://github.com/microsoft/DialoGPT) [GPT-Neo](https://github.com/EleutherAI/gpt-neo), [DistilBERT](https://arxiv.org/abs/1910.01108) fine-tuned on SQuAD, Codebert, XLNET, GPT-J)...
 - [Dhall](https://dhall-lang.org) and YAML config files, dhall for per-guild teams configurations
 - Multi-server streams notifications/trackers for twitch and goodgame.ru (using channels from dhall conf)
 - Tracking pull requests of watching by some user repositories on GitHub and posting detailed PR embeds
 - If lazy_static_models option is `False` conv model unloads after use (DialoGPT wait for 10-30 mins)
 - Veto helper (for banning maps against some player) using W3C statistics (`~veto` command)
 - Versus command showing score for one player against another for x seasons (`~vs` command)
 - Bets on live games with `~bet` and emojis under Live tracking games
 - Warcraft 3 commands `~stats`, `~today` and more with info from wacraft3.info and W3C ladder
 - Warcraft 3 hostbot API integration (creating games) using flo services and tonic for RPC
 - Some anti-spam protection (Free nitro scam and maybe more) using [this](https://raw.githubusercontent.com/nikolaischunk/discord-phishing-links/main/domain-list.json)
 - Query package atoms from [Zugaina](http://gpo.zugaina.org)
 - Some moderation automation, like timeout commands creating room for communicating with target user.
 - Slur words auto-removal / warnings system.
 - Using [Celery](https://github.com/rusty-celery/rusty-celery) and [RabbitMQ](https://www.rabbitmq.com) for distributed tasks queue.
 - Using tokio [UnixStream](https://docs.rs/tokio/1.17.0/tokio/net/struct.UnixStream.html#method.try_read_buf) on various sockets for IPC
 - Warcraft 3 Status Grid with active players on modes and weekly team players statistics.
 - [FloTV](https://w3flo.com/live) tokens generation using GraphQL API to [Flo Stats](https://stats.w3flo.com).
 - [Kafka](https://kafka.apache.org) interface for Vivaldi service to work with [Kalmarity](https://github.com/Miezhiko/Kalmarity)
 - [gpt4free](https://github.com/xtekky/gpt4free) interfaces on Vivaldi service using inline python under `catch_unwind`.
 - [OpenGPT](https://github.com/uesleibros/OpenGPT) interfaces on Vivaldi.
 - [poe-api](https://github.com/ading2210/poe-api) interface, reverse engineered wrapper for Quora's Poe
 - [g4f](https://gitler.moe/g4f) interfaces.

<img src="https://cdn.discordapp.com/attachments/249111029668249601/1025077275525382234/unknown.png">

## Cooking

 - Salieri needs [RabbitMQ](https://www.rabbitmq.com) to work properly
 - Vivaldi needs running [Kafka](https://kafka.apache.org)
 - Strauss needs to link with [PyTorch](https://pytorch.org/), instructions on [tch-rs](https://github.com/LaurentMazare/tch-rs)
 - Wagner needs `tokens.txt` file for poe API to work
 - to compile just use `cargo build --release` or `hake`
 - `cp conf.example.dhall conf.dhall` (initial constant options)
 - `cp conf.example.yml conf.yml` (those options may change in runtime)
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.dhall and fill `discord`, `tenor_key` and optionally `twitch` client data
 - conf.yml `twitch` value is OAuth access token, you can regenerate it with bot command `~twitch_token_update`

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
   , lazy_static_models   = False
   , gencache_on_start    = True
   , github_auth          = "AccountName:access_token"
   }
```

Optional Build Features:

 - `trackers` - enable games trackers / w3info news trackers / streams trackers
 - `flo` - gather flo nodes information / register players / host games
 - `full` - enable all

`cargo build --release --features flo,trackers` enabled by default,
use following to ignore non-needed features `cargo build --release --no-default-features`

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

## Development

 - **Schubert** common tiny helper library shared between all (almost)
 - **Wagner** python modules using inline-python
 - **Strauss** is set of tasks running on distributed tasks queue
 - **Amadeus** is discord bot service on Serenity
 - **Salieri** is celery daemon running on rabbitmq and processing tasks
 - **Vivaldi** is kafka daemon processing bert / network tasks
 - [Kalmarity](https://github.com/Miezhiko/Kalmarity) is separated bot using **Salieri** service

*rustfmt usage is forbidden*, *stylish-haskell is a must*, *pep8 is OK*

## Service

Amadeus service should run Salieri by itself, Vivaldi service is separated and optional, you probably just don't need it at all. (who am I talking with, after all)

```shell
cp misc/Salieri.service /etc/systemd/system/Salieri.service
cp misc/Vivaldi.service /etc/systemd/system/Vivaldi.service
cp misc/Amadeus.service /etc/systemd/system/Amadeus.service
systemctl daemon-reload
systemctl enable Amadeus
systemctl start Vivaldi
systemctl start Amadeus
```

it's safe to rebuild and restart it

```shell
systemctl restart Vivaldi
systemctl restart Amadeus
```

## Notes

 - Check [TODO.md](https://github.com/Miezhiko/Amadeus/blob/mawa/TODO.md) for planned work (also if you want to help me)
 - deepspeech mode was dropped (due bad voice to text model quality)
 - Code open sourced just for my friend Sirius to be able to see it, please don't pay [attention](https://www.youtube.com/watch?v=sKy6dSHn9Z8)
 - Fingon is cute
 - Additional thank to @fluxxu for removing my ban on W3C (this was kind of important for my motivation to improve flo/w3c aspects of Amadeus)
 - I feel sad (yes, still)
 - No I don't like classical music, just only Amadeus and Salieri was mentioned in Steins Gate and I needed more

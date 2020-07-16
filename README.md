Amadeus
=======

[![Build Status](https://travis-ci.org/Qeenon/Amadeus.svg?branch=master)](https://travis-ci.org/Qeenon/Amadeus)
[![CircleCI](https://circleci.com/gh/Qeenon/Amadeus.svg?style=shield)](https://circleci.com/gh/Qeenon/Amadeus)
[![Percentage of issues still open](http://isitmaintained.com/badge/open/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Percentage of issues still open")
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Average time to resolve an issue")
[![Open Source Helpers](https://www.codetriage.com/qeenon/amadeus/badges/users.svg)](https://www.codetriage.com/qeenon/amadeus)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.44.1-teal)
[![License: ISC](https://img.shields.io/badge/License-ISC-teal.svg)](https://opensource.org/licenses/ISC)
![Discord](https://img.shields.io/discord/611822838831251466?label=Discord)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=small)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_small)


<img align="right" src="https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png">

Memory storage and Artificial Intelligence system.

Built with Rust and Serenity Framework.

Features
--------

 - chatty (ability to change activity level on runtime)
 - many small useful commands (use `~help`)
 - using `Dhall` config files for nearly everything
 - live games tracking on w3champions
 - some more warcraft 3 related stuff, like player stats and news
 - points system on cannyls! https://github.com/frugalos/cannyls/wiki
 - runs async, using tokio https://tokio.rs
 - plays music streams (and actually good at it)

Preparing
---------

 - to compile just use `cargo build --release`
 - `cp conf.example.dhall conf.dhall` (initial constant options)
 - `cp conf.example.json conf.json` (those options may change in runtime)
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.ini and fill `token` and optionally `[Twitch]` section
 - highly suggested to fill `last_guild` or you will need to restart Amadeus to run background threads (things)

``` haskell
{ discord              = "put discord token here"
, guild                = 0
, twitch_oauth         = "Bearer AAAAAAAAAAAAAAAAAAAA"
, twitch_client_id     = "AAAAAAAAAAAAAAAAAAAAAAAA"
, twitch_client_secret = "AAAAAAAAAAAAAAAAAAAAA"
}
```

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

 - Do not block her
 - Only will work with server administrator permissions [WIP]


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_large)

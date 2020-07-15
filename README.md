Amadeus
=======

[![Build Status](https://travis-ci.org/Qeenon/Amadeus.svg?branch=master)](https://travis-ci.org/Qeenon/Amadeus)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_shield)
[![Percentage of issues still open](http://isitmaintained.com/badge/open/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Percentage of issues still open")
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/Qeenon/Amadeus.svg)](http://isitmaintained.com/project/Qeenon/Amadeus "Average time to resolve an issue")
[![Open Source Helpers](https://www.codetriage.com/qeenon/amadeus/badges/users.svg)](https://www.codetriage.com/qeenon/amadeus)
![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.44.1-teal)
[![License: ISC](https://img.shields.io/badge/License-ISC-teal.svg)](https://opensource.org/licenses/ISC)

Memory storage and artificial intelligence system.

Features
--------

 - fully async, runs on tokio https://tokio.rs
 - chatty (ability to change activity level on runtime)
 - many small useful commands
 - live games tracking on w3champoions
 - some more warcraft 3 related stuff, like player stats and news
 - points system on cannyls! https://github.com/frugalos/cannyls/wiki

Preparing
---------

 - to compile just use `cargo build --release`
 - `cp conf.ini.example conf.ini`
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.ini and fill `token` and optionally `[Twitch]` section
 - highly suggested to fill `last_guild` or you will need to restart Amadeus to run background threads (things)

``` ini
[Discord]
token=put token here

[Music]
rejoin=false
last_guild=0
last_channel=0
last_stream=0

[Twitch]
oauth=Bearer 0
client_id=0
client_secret=0
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
 - Only will work with server administrator permissions


## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2FQeenon%2FAmadeus.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2FQeenon%2FAmadeus?ref=badge_large)

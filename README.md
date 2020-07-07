Amadeus is the name of a memory storage and artificial intelligence system.
===========================================================================

To compile just use `cargo build --release`

Preparing
---------

 - `cp conf.ini.example conf.ini`
 - generate token here: https://discord.com/developers/applications
 - optionally for twitch support: https://dev.twitch.tv/docs/authentication
 - modify conf.ini and fill `token` and optionally `[Twitch]` section

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

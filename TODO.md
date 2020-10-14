- Activity level per server
- Possibly automatic twitch OAuth token regeneration every month (or better on error)
    alike curl -X POST "https://id.twitch.tv/oauth2/token?client_id=AAAAAAA&client_secret=QAAAAAA&grant_type=client_credentials"
- Service logs (alike `journalctl --since today -u Amadeus`)
- Maybe Lavalink for music instead of https://github.com/ytdl-org/youtube-dl
- Check for ngram generators?
- Split chain into 3 mods: cache, chain and possibly move general stuff to lib
- Ability to attach replays to #log games maybe
- Show region (EU/NA) on live matches
- Replace argparse to something less boring

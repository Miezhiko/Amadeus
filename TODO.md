- Activity level per server
- Possibly automatic twitch OAuth token regeneration every month (or better on error)
    alike curl -X POST "https://id.twitch.tv/oauth2/token?client_id=AAAAAAA&client_secret=QAAAAAA&grant_type=client_credentials"
- Store AI chain somewhere
- Function to get message content for chain
- meta.rs commands file is too big, split it
- Blacklist for pad games
- Pagination
- Service logs (alike `journalctl --since today -u Amadeus`)

use std::str;

use serenity::model::id::ChannelId;

pub static MAIN_CHANNEL: ChannelId    = ChannelId( 611822932897038341 );
pub static MIST_CHANNEL: ChannelId    = ChannelId( 827151604053835807 );
pub static SOLO_CHANNEL: ChannelId    = ChannelId( 721956117558853673 );
pub static TEAM2_CHANNEL: ChannelId   = ChannelId( 864417724445098004 );
pub static TEAM4_CHANNEL: ChannelId   = ChannelId( 864417767415349248 );
pub static STREAM_PICS: ChannelId     = ChannelId( 740153825272266822 );
pub static APM_PICS: ChannelId        = ChannelId( 752538491312930878 );
pub static MODERATION: ChannelId      = ChannelId( 740913303278321704 );

pub static GAME_CHANNELS: [&ChannelId; 3] =
  [ &SOLO_CHANNEL, &TEAM2_CHANNEL, &TEAM4_CHANNEL ];

pub static LIVE_ROLE: &str            = "🔴 LIVE";
pub static UNBLOCK_ROLE: &str         = "UNBLOCK AMADEUS";

pub static W3C_API: &str = "https://statistic-service.w3champions.com/api";

use std::str;

use serenity::model::id::{ ChannelId, MessageId };

pub const PREFIX: char                = '~';

pub const MAIN_CHANNEL: ChannelId     = ChannelId::new( 611822932897038341 );
pub const STREAM_PICS: ChannelId      = ChannelId::new( 740153825272266822 );
pub const APM_PICS: ChannelId         = ChannelId::new( 752538491312930878 );
pub const GITHUB_PRS: ChannelId       = ChannelId::new( 912241728243769395 );
pub const MAIN_LOG: ChannelId         = ChannelId::new( 740913303278321704 );

pub const LIVE_ROLE: &str             = "🔴 LIVE";
pub const UNBLOCK_ROLE: &str          = "UNBLOCK AMADEUS";
pub const MUTED_ROLE: &str            = "muted";

// TODO: move this into teams configs
pub const MUTED_ROOMS: &[ChannelId]   = &[ ChannelId::new( 1093531555642744922 ) ];

pub const W3C_STATS_ROOM: ChannelId   = ChannelId::new( 965968135666696322 );
pub const W3C_STATS_MSG: MessageId    = MessageId::new( 965968232328609802 );
pub const W3C_STATS_MSG2: MessageId   = MessageId::new( 993816096375324742 );

pub const W3C_API: &str = "https://website-backend.w3champions.com/api";

#[cfg(feature = "flo")]
pub const FLO_API: &str = "tcp://service.w3flo.com:3549";

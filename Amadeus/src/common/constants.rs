use std::str;

use serenity::model::id::{ ChannelId, MessageId };

pub const PREFIX: char                = '~';

pub const MAIN_CHANNEL: ChannelId     = ChannelId( 611822932897038341 );
pub const STREAM_PICS: ChannelId      = ChannelId( 740153825272266822 );
pub const APM_PICS: ChannelId         = ChannelId( 752538491312930878 );
pub const GITHUB_PRS: ChannelId       = ChannelId( 912241728243769395 );

pub const LIVE_ROLE: &str             = "🔴 LIVE";
pub const UNBLOCK_ROLE: &str          = "UNBLOCK AMADEUS";
pub const MUTED_ROLE: &str            = "muted";

pub const MUTED_ROOMS: &[ChannelId]   = &[ ChannelId( 958705907099918386 )
                                         , ChannelId( 958712754951323718 ) ];

pub const W3C_STATS_ROOM: ChannelId   = ChannelId( 965968135666696322 );
pub const W3C_STATS_MSG: MessageId    = MessageId( 965968232328609802 );

pub const W3C_API: &str = "https://website-backend.w3champions.com/api";

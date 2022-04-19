use std::str;

use serenity::model::id::ChannelId;

pub static PREFIX: char               = '~';

pub static MAIN_CHANNEL: ChannelId    = ChannelId( 611822932897038341 );
pub static STREAM_PICS: ChannelId     = ChannelId( 740153825272266822 );
pub static APM_PICS: ChannelId        = ChannelId( 752538491312930878 );
pub static GITHUB_PRS: ChannelId      = ChannelId( 912241728243769395 );

pub static LIVE_ROLE: &str            = "🔴 LIVE";
pub static UNBLOCK_ROLE: &str         = "UNBLOCK AMADEUS";
pub static MUTED_ROLE: &str           = "muted";

pub static MUTED_ROOMS: &[ChannelId]  = &[ ChannelId( 958705907099918386 )
                                         , ChannelId( 958712754951323718 ) ];

pub static W3C_API: &str = "https://website-backend.w3champions.com/api";

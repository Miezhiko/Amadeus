use std::{
  str,
  num::NonZeroU64
};

use serenity::model::id::{ ChannelId, MessageId };

pub const PREFIX: char                = '~';

pub const MAIN_CHANNEL: ChannelId     = ChannelId( NonZeroU64::new( 611822932897038341 ).unwrap() );
pub const STREAM_PICS: ChannelId      = ChannelId( NonZeroU64::new( 740153825272266822 ).unwrap() );
pub const APM_PICS: ChannelId         = ChannelId( NonZeroU64::new( 752538491312930878 ).unwrap() );
pub const GITHUB_PRS: ChannelId       = ChannelId( NonZeroU64::new( 912241728243769395 ).unwrap() );
pub const MAIN_LOG: ChannelId         = ChannelId( NonZeroU64::new( 740913303278321704 ).unwrap() );

pub const LIVE_ROLE: &str             = "🔴 LIVE";
pub const UNBLOCK_ROLE: &str          = "UNBLOCK AMADEUS";
pub const MUTED_ROLE: &str            = "muted";

pub const MUTED_ROOMS: &[ChannelId]   = &[ ChannelId( NonZeroU64::new( 958705907099918386 ).unwrap() )
                                         , ChannelId( NonZeroU64::new( 958712754951323718 ).unwrap() ) ];

pub const W3C_STATS_ROOM: ChannelId   = ChannelId( NonZeroU64::new( 965968135666696322 ).unwrap() );
pub const W3C_STATS_MSG: MessageId    = MessageId( NonZeroU64::new( 965968232328609802 ).unwrap() );
pub const W3C_STATS_MSG2: MessageId   = MessageId( NonZeroU64::new( 993816096375324742 ).unwrap() );

pub const W3C_API: &str = "https://website-backend.w3champions.com/api";

#[cfg(feature = "flo")]
pub const FLO_API: &str = "tcp://service.w3flo.com:3549";

use serenity::{
  prelude::*,
  model::{
    channel::GuildChannel,
    id::ChannelId
  }
};

use std::collections::HashMap;

use futures_util::stream::{self, StreamExt};

pub async fn channel_by_name<'a>( ctx: &Context
                            , channels: &'a HashMap<ChannelId, GuildChannel>
                            , channel_name: &str) -> Option<(&'a ChannelId, &'a GuildChannel)> {
  let some_channels = stream::iter(channels.iter())
      .filter_map(|(c, cx)| async move {
        if let Some(name) = c.name(&ctx).await {
          if name == channel_name { Some((c, cx)) } else { None }
        } else { None }
      }).collect::<Vec<(&ChannelId, &GuildChannel)>>().await;
  if some_channels.len() > 0 {
    Some(some_channels[0])
  } else {
    None
  }
}

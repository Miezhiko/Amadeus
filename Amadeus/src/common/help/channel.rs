use serenity::{
  prelude::*,
  model::{ channel::GuildChannel
         , id::ChannelId }
};

use std::collections::HashMap;

use futures_util::stream::{ self, StreamExt };

pub async fn channel_by_name<'a>( ctx: &'a Context
                                , channels: &'a HashMap<ChannelId, GuildChannel>
                                , channel_name: &'a str
                                ) -> Option<(&'a ChannelId, &'a GuildChannel)> {
  let some_channels = stream::iter(channels.iter())
    .filter_map(|(c, cx)| async move {
      if let Ok(name) = c.name(&ctx).await {
        if name == channel_name { Some((c, cx)) } else { None }
      } else { None }
    }).collect::<Vec<(&ChannelId, &GuildChannel)>>().await;
  if some_channels.is_empty() {
    None
  } else {
    Some(some_channels[0])
  }
}

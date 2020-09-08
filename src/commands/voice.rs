use crate::{
  types::options::ROptions,
  common::{
    msg::{ direct_message, reply },
    options
  }
};

use serenity::{
  model::{ misc::Mentionable
         , id::GuildId, id::ChannelId
         , channel::* },
  client::{ bridge::voice::ClientVoiceManager },
  voice,
  prelude::*,
  framework::standard::{
    Args, Delimiter, CommandResult,
    macros::command
  }
};

use std::sync::Arc;

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
  type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub async fn rejoin_voice_channel(ctx : &Context, conf: &ROptions) {
  if conf.rejoin && conf.last_guild != 0 && conf.last_channel != 0 {
    set!{ last_guild_conf   = GuildId( conf.last_guild )
        , last_channel_conf = ChannelId( conf.last_channel ) };
    let manager_lock =
      ctx.data.read().await
        .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock().await;
    if manager.join(last_guild_conf, last_channel_conf).is_some() {
      info!("Rejoined voice channel: {}", last_channel_conf);
      if conf.last_stream != "" {
        if let Some(handler) = manager.get_mut(last_guild_conf) {
          match voice::ytdl(&conf.last_stream).await {
            Ok(source) => handler.play(source),
            Err(why)   => error!("Err starting source: {:?}", why)
          };
        }
      }
    } else {
      error!("Failed to rejoin voice channel: {}", last_channel_conf);
    }
  }
}

#[command]
#[description("join voice channel")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
  let guild = match msg.guild(&ctx).await {
    Some(guild) => guild,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    }
  };
  let guild_id = guild.id;
  let channel_id = guild
    .voice_states.get(&msg.author.id)
    .and_then(|voice_state| voice_state.channel_id);
  let connect_to = match channel_id {
    Some(channel) => channel,
    None => {
      reply(ctx, msg, "You're not in a voice channel").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
    .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if manager.join(guild_id, connect_to).is_some() {
    let mut opts = options::get_roptions().await?;
    if opts.last_guild != guild_id.0
    || opts.last_channel != connect_to.0
    || !opts.rejoin {
      opts.rejoin = true;
      opts.last_guild = guild_id.0;
      opts.last_channel = connect_to.0;
      options::put_roptions(&opts).await?;
    }
    if let Err(why) = msg.channel_id.say(&ctx, &format!("I've joined {}", connect_to.mention())).await {
      error!("failed to say joined {:?}", why);
    }
  } else {
    direct_message(ctx, msg, "Some error joining the channel...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("leave voice channel")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      direct_message(ctx, msg, "Groups and DMs not supported").await;
      return Ok(());
    },
  };
  let manager_lock = ctx.data.read()
      .await.get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  let has_handler = manager.get(guild_id).is_some();
  if has_handler {
    manager.remove(guild_id);
    let _ = msg.channel_id.say(&ctx, "I left voice channel");
    let mut conf = options::get_roptions().await?;
    if conf.rejoin {
      conf.rejoin = false;
      options::put_roptions(&conf).await?;
    }
  } else {
    reply(ctx, &msg, "I'm not in a voice channel").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("play some mp3/youtube stream")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let url =
    if !args.is_empty() {
      match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
          reply(ctx, msg, "You must provide a URL to a video or audio").await;
          return Ok(());
        }
      }
    } else {
      let conf = options::get_roptions().await?;
      conf.last_stream
    };
  if !url.starts_with("http") {
    reply(ctx, msg, "You must provide a valid URL").await;
    return Ok(());
  }
  let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
    Some(channel) => channel.guild_id,
    None => {
      reply(ctx, msg, "Error finding channel info...").await;
      return Ok(());
    }
  };
  let manager_lock = ctx.data.read().await
      .get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
  let mut manager = manager_lock.lock().await;
  if let Some(handler) = manager.get_mut(guild_id) {
    let source = match voice::ytdl(&url).await {
      Ok(source) => source,
      Err(why) => {
        error!("Err starting source: {:?}", why);
        reply(ctx, msg, &format!("Sorry, error sourcing ffmpeg {:?}", why)).await;
        return Ok(());
      }
    };
    handler.play_only(source);
    let mut conf = options::get_roptions().await?;
    let last_stream_conf = conf.last_stream;
    if last_stream_conf != url {
      conf.last_stream = url.clone();
      options::put_roptions(&conf).await?;
    }
    reply(ctx, msg, &format!("playing stream: {}", url)).await;
  } else {
    reply(ctx, msg, "Not in a voice channel to play in...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  Ok(())
}

#[command]
#[description("play last stream again")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
  play(ctx, msg, Args::new("", &[Delimiter::Single(' ')])).await
}

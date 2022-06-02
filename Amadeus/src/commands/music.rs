use crate::{
  types::options::ROptions,
  common::{
    msg::{ direct_message, reply },
    options
  }
};

#[cfg(feature = "voice_analysis")]
use crate::common::voice_analysis::*;
#[cfg(feature = "voice_analysis")]
use songbird::CoreEvent;

#[cfg(feature = "voice_analysis")]
use std::sync::Arc;

use serenity::{
  prelude::*,
  model::{
    mention::Mentionable,
    id::GuildId, id::ChannelId,
    user::User, guild::Guild,
    channel::*
  },
  framework::standard::{
    CommandResult, Args,
    macros::command,
    Delimiter
  }
};

pub async fn rejoin_voice_channel(ctx: &Context, conf: &ROptions) {
  if conf.rejoin && conf.last_guild != 0 && conf.last_channel != 0 {
    set!{ last_guild_conf   = GuildId( conf.last_guild )
        , last_channel_conf = ChannelId( conf.last_channel ) };

    let manager = songbird::get(ctx).await
      .expect("Songbird Voice client placed in at initialisation.").clone();

    let (_call, j) = manager.join(last_guild_conf, last_channel_conf).await;

    if j.is_ok() {
      info!("Rejoined voice channel: {last_channel_conf}");
      if !conf.last_stream.is_empty() {
        if let Some(handler_lock) = manager.get(last_guild_conf) {
          let mut handler = handler_lock.lock().await;
          match songbird::ytdl(&conf.last_stream).await {
            Ok(source) => { handler.play_source(source); },
            Err(why)   => { error!("Err starting source {why}"); }
          };
        }
      }
    } else {
      error!("Failed to rejoin voice channel: {last_channel_conf}");
    }
  }
}

#[command]
#[only_in("guilds")]
#[description("join voice channel")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
  let voice_states = match msg.guild(ctx.cache.as_ref()) {
    Some(guild) => guild.voice_states.clone(),
    None => { return Ok(()); }
  };
  let guild_id = msg.guild_id.unwrap_or_default();
  let channel_id = voice_states
    .get(&msg.author.id)
    .and_then(|voice_state| voice_state.channel_id);
  let connect_to = match channel_id {
    Some(channel) => channel,
    None => {
      reply(ctx, msg, "You're not in a voice channel").await;
      return Ok(());
    }
  };
  let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
  let (_handler_lock, conn_result) = manager.join(guild_id, connect_to).await;
  if conn_result.is_ok() {
    let mut opts = options::get_roptions().await?;
    if opts.last_guild != guild_id.0
    || opts.last_channel != connect_to.0
    || !opts.rejoin {
      opts.rejoin = true;
      opts.last_guild = guild_id.0;
      opts.last_channel = connect_to.0;
      options::put_roptions(&opts).await?;
    }

    #[cfg(feature = "voice_analysis")]
    {
      let ctx_arc = Arc::new(ctx.clone());
      let receiver = Receiver::new(ctx_arc);
      let mut handler = _handler_lock.lock().await;

      handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
      handler.add_global_event(CoreEvent::SpeakingUpdate.into(), receiver.clone());
      handler.add_global_event(CoreEvent::VoicePacket.into(), receiver.clone());
      handler.add_global_event(CoreEvent::ClientConnect.into(), receiver.clone());
      handler.add_global_event(CoreEvent::ClientDisconnect.into(), receiver.clone());
    }

    if let Err(why) = msg.channel_id.say(ctx, &format!("I've joined {}", connect_to.mention())).await {
      error!("failed to say joined {why}");
    }
  } else {
    direct_message(ctx, msg, "Some error joining the channel...").await;
  }
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

pub async fn join_slash(ctx: &Context, user: &User, guild: &Guild) -> anyhow::Result<()> {
  let guild_id = guild.id;
  let channel_id = guild
    .voice_states.get(&user.id)
    .and_then(|voice_state| voice_state.channel_id);
  let connect_to = match channel_id {
    Some(channel) => channel,
    None => {
      if let Err(why) = user.dm(ctx, |m| m.content("You're not in a voice channel")).await {
        error!("Error DMing user: {why}");
      }
      return Ok(());
    }
  };
  let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
  let (_handler_lock, conn_result) = manager.join(guild_id, connect_to).await;
  if conn_result.is_ok() {
    let mut opts = options::get_roptions().await?;
    if opts.last_guild != guild_id.0
    || opts.last_channel != connect_to.0
    || !opts.rejoin {
      opts.rejoin = true;
      opts.last_guild = guild_id.0;
      opts.last_channel = connect_to.0;
      options::put_roptions(&opts).await?;
    }

    #[cfg(feature = "voice_analysis")]
    {
      let ctx_arc = Arc::new(ctx.clone());
      let receiver = Receiver::new(ctx_arc);
      let mut handler = _handler_lock.lock().await;

      handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
      handler.add_global_event(CoreEvent::SpeakingUpdate.into(), receiver.clone());
      handler.add_global_event(CoreEvent::VoicePacket.into(), receiver.clone());
      handler.add_global_event(CoreEvent::ClientConnect.into(), receiver.clone());
      handler.add_global_event(CoreEvent::ClientDisconnect.into(), receiver.clone());
    }

  } else {
    error!("Some error joining the channel on slash command");
  }
  Ok(())
}

#[command]
#[description("leave voice channel")]
pub async fn leave(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
  let guild_id = match ctx.cache.guild_channel(msg.channel_id) {
    Some(channel) => channel.guild_id,
    None => { return Ok(()); }
  };
  let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.")
    .clone();
  let has_handler = manager.get(guild_id).is_some();
  if has_handler {
    if let Err(why) = manager.remove(guild_id).await {
      error!("Error removing songbird manager from guild: {why}");
    }
    let _ = msg.channel_id.say(&ctx, "I left voice channel");
    let mut conf = options::get_roptions().await?;
    if conf.rejoin {
      conf.rejoin = false;
      options::put_roptions(&conf).await?;
    }
  } else {
    reply(ctx, msg, "I'm not in a voice channel").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[description("play some mp3/youtube stream")]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
  let guild_id = match ctx.cache.guild_channel(msg.channel_id) {
    Some(channel) => channel.guild_id,
    None => { return Ok(()); }
  };
  let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
  if let Some(handler_lock) = manager.get(guild_id) {
    let mut handler = handler_lock.lock().await;
    let source = match songbird::ytdl(&url).await {
      Ok(source) => source,
      Err(why) => {
        error!("Err starting source {why}");
        reply(ctx, msg, &format!("Sorry, error sourcing ffmpeg {why}")).await;
        return Ok(());
      }
    };
    handler.play_source(source);
    let mut conf = options::get_roptions().await?;
    let last_stream_conf = conf.last_stream;
    if last_stream_conf != url {
      conf.last_stream = url.clone();
      options::put_roptions(&conf).await?;
    }
    reply(ctx, msg, &format!("playing stream: {url}")).await;
  } else {
    reply(ctx, msg, "Not in a voice channel to play in...").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[description("play last stream again")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
  play(ctx, msg, Args::new("", &[Delimiter::Single(' ')])).await
}

use crate::{
  types::{ options::ROptions
         , serenity::ReqwestClient },
  common::{ msg::reply
          , options }
};

use songbird::input::YoutubeDl;

use std::sync::Arc;

use serenity::{
  prelude::*,
  builder::CreateMessage,
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
    set!{ last_guild_conf   = GuildId( to_nzu!( conf.last_guild ) )
        , last_channel_conf = ChannelId( to_nzu!( conf.last_channel ) ) };

    let reqwest_client = {
      let data = ctx.data.read().await;
      data.get::<ReqwestClient>().unwrap().clone()
    };

    let manager = songbird::get(ctx).await
      .expect("Songbird Voice client placed in at initialisation.").clone();

    match manager.join(last_guild_conf, last_channel_conf).await {
      Ok (_call) => {
        info!("Rejoined voice channel: {last_channel_conf}");
        if !conf.last_stream.is_empty() {
          if let Some(handler_lock) = manager.get(last_guild_conf) {
            let mut handler = handler_lock.lock().await;
            let youtube = YoutubeDl::new(
              Arc::unwrap_or_clone( reqwest_client ),
              conf.last_stream.clone());
            handler.play_input(youtube.into());
          }
        }
      } , Err (err) => {
        error!("JoinError on rejoin: {err}");
      }
    };
  }
}

#[command]
#[bucket = "A"]
#[only_in("guilds")]
#[description("join voice channel")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
  let voice_states = match msg.guild(ctx.cache.as_ref()) {
    Some(guild) => guild.voice_states.clone(),
    None => { return Ok(()); }
  };
  let guild_id = msg.guild_id.unwrap();
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
  let _call = manager.join(guild_id, connect_to).await?;

  let mut opts = options::get_roptions().await?;

  if opts.last_guild != guild_id.0.get()
  || opts.last_channel != connect_to.0.get()
  || !opts.rejoin {
    opts.rejoin = true;
    opts.last_guild = guild_id.0.get();
    opts.last_channel = connect_to.0.get();
    options::put_roptions(&opts).await?;
  }

  if let Err(why) = msg.channel_id.say(ctx, &format!("I've joined {}", connect_to.mention())).await {
    error!("failed to say joined {why}");
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
      if let Err(why) = user.dm(ctx, CreateMessage::new().content("You're not in a voice channel")).await {
        error!("Error DMing user: {why}");
      }
      return Ok(());
    }
  };
  let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
  let _call = manager.join(guild_id, connect_to).await?;

  let mut opts = options::get_roptions().await?;

  if opts.last_guild != guild_id.0.get()
  || opts.last_channel != connect_to.0.get()
  || !opts.rejoin {
    opts.rejoin = true;
    opts.last_guild = guild_id.0.get();
    opts.last_channel = connect_to.0.get();
    options::put_roptions(&opts).await?;
  }

  Ok(())
}

#[command]
#[bucket = "A"]
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
    msg.channel_id.say(&ctx, "I left voice channel").await?;
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
#[bucket = "A"]
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
    let reqwest_client = {
      let data = ctx.data.read().await;
      data.get::<ReqwestClient>().unwrap().clone()
    };
    let mut handler = handler_lock.lock().await;
    let youtube = YoutubeDl::new(
      Arc::unwrap_or_clone( reqwest_client ),
      url.clone());
    handler.play_input(youtube.into());
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
#[bucket = "A"]
#[description("play last stream again")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
  play(ctx, msg, Args::new("", &[Delimiter::Single(' ')])).await
}

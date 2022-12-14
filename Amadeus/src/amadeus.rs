use crate::{
  types::{
    serenity::{ PubCreds, ReqwestClient
              , CoreGuild, CoreGuilds, IContext
              , IServer, AllGuilds },
    options::IOptions
  },
  common::{ constants::PREFIX
          , system::ShardManagerContainer
          , voice::DECODE_TYPE },
  handler::Handler,
  groups::*, hooks::*
};

#[cfg(feature = "flo")]
use crate::steins::warcraft::flo::FLO_SECRET;

use songbird::{ Config as DriverConfig
              , driver::CryptoMode
              , SerenityInit, Songbird };

use serenity::{
  framework::StandardFramework,
  model::{ gateway::GatewayIntents
         , id::ApplicationId }
};

use tracing::{ Level, instrument };
use tracing_subscriber::FmtSubscriber;
use tracing_log::LogTracer;

use std::{ collections::{ HashSet, HashMap }
         , sync::Arc };

#[instrument]
pub async fn run(opts: IOptions) ->
  anyhow::Result<(), Box<dyn std::error::Error + Send + Sync>> {

  LogTracer::init()?;
  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;

  info!("Amadeus {}", env!("CARGO_PKG_VERSION").to_string());

  let http = serenity::http::Http::new(&opts.discord);

  info!("http context created");

  let (owners, amadeus_id) = match http.get_current_application_info().await {
    Ok(info) => {
      let mut owners = HashSet::new();
      if let Some(team) = info.team {
        owners.insert(team.owner_user_id);
      } else {
        owners.insert(info.owner.id);
      }
      match http.get_current_user().await {
        Ok(bot_id) => (owners, bot_id.id),
        Err(why) => panic!("Could not access the bot id: {why}")
      }
    },
    Err(why) => panic!("Could not access application info: {why}")
  };

  info!("application info loaded");

  let mut creds = HashMap::new();
  creds.insert("tenor".to_string(), opts.tenor_key.clone());

  #[cfg(feature = "trackers")]
  {
    creds.insert("twitch_client".to_string(), opts.twitch_client_id.clone());
    creds.insert("twitch_secret".to_string(), opts.twitch_client_secret.clone());
  }

  #[cfg(feature = "flo")]
  {
    if let Err(what) = FLO_SECRET.set(opts.flo_secret.clone()) {
      error!("failed to set flo secret {what}");
    }
  }

  let mut core_guilds = HashMap::new();
  core_guilds.insert(CoreGuild::HEmo, opts.guild);
  core_guilds.insert(CoreGuild::Storage, opts.amadeus_guild);
  core_guilds.insert(CoreGuild::Amadeus, amadeus_id.0.get());

  let mut all_guilds = opts.servers.clone();
  all_guilds.push( IServer { id: opts.guild, kind: CoreGuild::HEmo } );
  all_guilds.push( IServer { id: opts.amadeus_guild, kind: CoreGuild::Storage } );

  let context = IContext { lazy_static_models: opts.lazy_static_models };

  // mut is used for optional groups
  #[allow(unused_mut)]
  let mut std_framework =
    StandardFramework::new()
      .before(before)
      .after(after)
      .unrecognised_command(unrecognised_command)
      .group(&META_GROUP)
      .group(&CHAT_GROUP)
      .group(&IMAGES_GROUP)
      .group(&WARCRAFT_GROUP)
      .group(&PAD_GROUP)
      .group(&INFO_GROUP)
      .group(&OWNER_GROUP)
      .group(&MODERATOR_GROUP)
      .group(&GENTOO_GROUP)
      .help(&HELP_COMMAND);

  std_framework.configure(|c| c
    .owners(owners)
    .on_mention(Some(amadeus_id))
    .prefix(PREFIX)
    .delimiters(vec![" ", ";", "\n", "\t"])
    .case_insensitivity(true)
  );

  #[cfg(not(target_os = "windows"))]
  {
    std_framework = std_framework.group(&TRANSLATE_GROUP)
  }

  #[cfg(feature = "flo")]
  {
    std_framework = std_framework.group(&FLO_GROUP)
  }

  let songbird = Songbird::serenity();
  songbird.set_config(
    DriverConfig::default()
      .decode_mode(DECODE_TYPE)
      .crypto_mode(CryptoMode::Normal),
  );
  let intents = GatewayIntents::GUILDS
              | GatewayIntents::GUILD_MEMBERS
              | GatewayIntents::GUILD_BANS
              | GatewayIntents::GUILD_PRESENCES
              | GatewayIntents::GUILD_MESSAGES
              | GatewayIntents::GUILD_MESSAGE_REACTIONS
              | GatewayIntents::GUILD_WEBHOOKS
              | GatewayIntents::GUILD_VOICE_STATES
              | GatewayIntents::MESSAGE_CONTENT;
  let mut client =
    serenity::Client::builder(&opts.discord, intents)
      .application_id( ApplicationId(to_nzu!(opts.app_id)) )
      .event_handler(Handler::new( opts
                                 , amadeus_id
                                 ).await?
                    )
      .framework(std_framework)
      .register_songbird_with(songbird).await?;
  {
    let mut data = client.data.write().await;
    let request_client = reqwest::Client::builder()
                                .pool_max_idle_per_host(0)
                                .build()?;
    data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    data.insert::<ReqwestClient>(Arc::new(request_client));
    data.insert::<PubCreds>(Arc::new(creds));
    data.insert::<CoreGuilds>(Arc::new(core_guilds));
    data.insert::<AllGuilds>(Arc::new(all_guilds));
    data.insert::<IContext>(Arc::new(context));
  }

  // start listening for events by starting a single shard
  if let Err(why) = client.start_autosharded().await {
    eprintln!("An error occurred while running the client: {why}");
  }

  Ok(())
}

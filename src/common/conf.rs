const CONF_FILE_NAME: &'static str = "conf.ini";

use crate::common::types;

use ini::Ini;

pub fn write_config(opts : &types::AOptions) {
  let mut conf = Ini::new();
  conf.with_section(None::<String>)
    .set("encoding", "utf-8");
  conf.with_section(Some("Discord".to_owned()))
    .set("token", opts.discord.as_str());
  conf.with_section(Some("Music".to_owned()))
    .set("rejoin", if opts.rejoin { "true" } else { "false" })
    .set("last_guild", opts.last_guild.as_str())
    .set("last_channel", opts.last_channel.as_str())
    .set("last_stream", opts.last_stream.as_str());
  conf.with_section(Some("Twitch".to_owned()))
    .set("oauth", opts.twitch_oauth.as_str())
    .set("client_id", opts.twitch_client_id.as_str())
    .set("client_secret", opts.twitch_client_id.as_str());
  conf.write_to_file(CONF_FILE_NAME).unwrap();
}

pub fn parse_config() -> types::AOptions {
  let mut options: types::AOptions = types::AOptions {
    rejoin:               true,
    discord:              String::from(""),
    last_guild:           String::from(""),
    last_channel:         String::from(""),
    last_stream:          String::from(""),
    twitch_oauth:         String::from(""),
    twitch_client_id:     String::from(""),
    twitch_client_secret: String::from("")
  };
  let config_load_status =
    Ini::load_from_file(CONF_FILE_NAME)
      .and_then(|conf| Ok({
        options.discord               = conf["Discord"]["token"].to_owned();
        options.rejoin                = &(conf["Music"]["rejoin"]) == "true";
        options.last_guild            = conf["Music"]["last_guild"].to_owned();
        options.last_channel          = conf["Music"]["last_channel"].to_owned();
        options.last_stream           = conf["Music"]["last_stream"].to_owned();
        options.twitch_oauth          = conf["Twitch"]["oauth"].to_owned();
        options.twitch_client_id      = conf["Twitch"]["client_id"].to_owned();
        options.twitch_client_secret  = conf["Twitch"]["client_secret"].to_owned();
      }));
  if config_load_status.is_err() {
    write_config(&options);
  }
  options
}

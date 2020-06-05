#[derive(Debug, Clone)]
pub struct AOptions {
  pub rejoin: bool,
  pub discord: String,
  pub last_guild: String,
  pub last_channel: String,
  pub last_stream: String,
  pub last_channel_chat: String,
  pub twitch_oauth: String,
  pub twitch_client_id: String,
  pub twitch_client_secret: String
}

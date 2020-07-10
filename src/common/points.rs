use tokio::task::{ self, JoinError };

pub async fn add_points<'a>( _guild_id: u64
                           , _user_id: u64
                           , _points: u64) {

}

pub async fn get_points(_guild_id: u64, _user_id: u64) -> Result<u64, JoinError> {
  task::spawn_blocking(move || {
    0
  }).await
}

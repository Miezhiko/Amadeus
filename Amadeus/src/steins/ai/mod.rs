pub mod boris;
pub mod uwu;
pub mod bert;
pub mod cache;
pub mod response;
pub mod chain;

pub async fn reinit() {
  cache::reinit().await;
  mozart::bert::chat::reinit().await;
}

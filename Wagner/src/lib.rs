#[macro_use] extern crate anyhow;

mod personality;
pub mod poe;
pub mod gpt4free;
pub mod opengpt;

pub async fn wagner(msg: &str) -> anyhow::Result<String> {
  let mut fmode = true;
  if msg.contains("please") || msg.contains("пожалуйста") {
    fmode = false;
  } else if msg.contains("Please")
         || msg.contains("Пожалуйста")
         || msg.contains("PLEASE") {
    if let Ok(gpt4free_result) = poe::generate( msg ) {
      return Ok(gpt4free_result)
    } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( msg ) {
      return Ok(gpt4free_result)
    }
    fmode = false;
  }

  if let Ok(gpt4free_result)        = gpt4free::useless::generate( msg, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::aiassist::generate( msg, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::deepai::generate( msg, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::gptworldAi::generate( msg, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::italygpt::generate( msg, fmode ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::theb::generate( msg ) {
    Ok(gpt4free_result)
  } else { Err(anyhow!("Failed to generate Wagner response")) }
}

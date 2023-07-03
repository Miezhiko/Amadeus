#[macro_use] extern crate anyhow;

mod personality;
pub mod poe;
pub mod gpt4free;
pub mod opengpt;
pub mod g4f;
pub mod chimera;

pub fn poe_generate(msg: &str) -> anyhow::Result<String> {
  if let Ok(gpt4free_result)        = poe::generate( msg, "beaver" ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg, "chinchilla" ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg, "capybara" ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg, "a2_100k" ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg, "a2_2" ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe::generate( msg, "a2" ) {
    Ok(gpt4free_result)
  } else {
    Err(anyhow!("Failed to generate poe response"))
  }
}

pub async fn generate(msg: &str) -> anyhow::Result<String> {
  if let Ok(gpt4free_result)        = chimera::generate( msg, false, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::phind::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe_generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::deepai::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::aiassist::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::gptworldAi::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::italygpt::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::forefront::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::getgpt::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::aichat::generate( msg, true, "Amadeus" ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::yqcloud::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::theb::generate( msg ) {
    Ok(gpt4free_result)
  } else { Err(anyhow!("Failed to generate Wagner response")) }
}

pub async fn wagner(msg: &str, bot_name: &str) -> anyhow::Result<String> {
  let mut fmode = true;
  if msg.contains("please") || msg.contains("пожалуйста") {
    fmode = false;
  } else if msg.contains("Please")
         || msg.contains("Пожалуйста")
         || msg.contains("PLEASE") {
    if let Ok(gpt4free_result) = chimera::generate( msg, false, bot_name ).await {
      return Ok(gpt4free_result)
    } else if let Ok(gpt4free_result) = g4f::phind::generate( msg ) {
      return Ok(gpt4free_result)
    } else if let Ok(gpt4free_result) = poe_generate( msg ) {
      return Ok(gpt4free_result)
    } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( msg ) {
      return Ok(gpt4free_result)
    }
    fmode = false;
  }

  if let Ok(gpt4free_result)     = chimera::generate( msg, fmode, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::deepai::generate( msg, fmode, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::gptworldAi::generate( msg, fmode, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::italygpt::generate( msg, fmode, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::phind::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::aiassist::generate( msg, fmode, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::forefront::generate( msg, true, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::getgpt::generate( msg, true, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::aichat::generate( msg, true, bot_name ).await {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = g4f::yqcloud::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = gpt4free::theb::generate( msg ) {
    Ok(gpt4free_result)
  } else if let Ok(gpt4free_result) = poe_generate( msg ) {
    Ok(gpt4free_result)
  } else { Err(anyhow!("Failed to generate Wagner response")) }
}

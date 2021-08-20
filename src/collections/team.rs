use crate::types::team::{ DiscordServer, DiscordPlayer };

use once_cell::sync::Lazy;

static HEMOD: &str   = "dhall/team/hemo.dhall";
static RAVENSD: &str = "dhall/team/ravens.dhall";

pub static HEMO: Lazy<DiscordServer>    = Lazy::new(|| dhall!(HEMOD));
pub static RAVENS: Lazy<DiscordServer>  = Lazy::new(|| dhall!(RAVENSD));

fn get_discord_players() -> Vec<DiscordPlayer> {
  let mut discord_players = vec![];
  for disc in &[&HEMO, &RAVENS] {
    for player in disc.players.iter() {
      if let Some(existing) =
        discord_players.iter_mut()
                       .find(|dp: &&mut DiscordPlayer| (**dp).player.battletag == player.battletag) {
        existing.discords.push(disc.uid);
      } else {
        let discord_player = DiscordPlayer{ player: player.clone()
                                          , discords: vec![disc.uid] };
        discord_players.push(discord_player);
      }
    }
  }
  discord_players
}

pub static PLAYERS: Lazy<Vec<DiscordPlayer>> = Lazy::new(|| get_discord_players());

#[cfg(test)]
mod stuff_dhall_tests {
  use super::*;
  fn dhall_players(f: &str) -> Result<(), String> {
    match serde_dhall::from_file(f).parse::<DiscordServer>() {
      Ok(_) => Ok(()),
      Err(de) => Err(format!("Failed to parse {:?}", de))
    }
  }
  #[test]
  fn hemo() -> Result<(), String> { dhall_players(HEMOD) }
  #[test]
  fn ravens() -> Result<(), String> { dhall_players(RAVENSD) }
  #[test]
  fn players() -> Result<(), String> { 
    let discord_players = get_discord_players();
    if discord_players.is_empty() {
      Err("Can't get players :(".into())
    } else {
      Ok(())
    }
  }
}

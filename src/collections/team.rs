use crate::types::team::{Discords, DiscordFields, DiscordPlayer, DiscordServer};

use once_cell::sync::Lazy;

static HEMOD: &str   = "dhall/team/hemo.dhall";
static RAVENSD: &str = "dhall/team/ravens.dhall";
static NEEJITD: &str = "dhall/team/neejit.dhall";

pub static HEMO: Lazy<DiscordServer>    = Lazy::new(|| dhall!(HEMOD));
pub static RAVENS: Lazy<DiscordServer>  = Lazy::new(|| dhall!(RAVENSD));
pub static NEEJIT: Lazy<DiscordServer>  = Lazy::new(|| dhall!(NEEJITD));

fn get_discord_servers() -> Discords {
  let mut discord_servers: Discords = Discords::new();
  for disc in &[&HEMO, &RAVENS, &NEEJIT] {
    let discord = DiscordFields
                    { games:    disc.games
                    , games2:   disc.games2
                    , games4:   disc.games4
                    , streams:  disc.streams
                    , events:   disc.events };
    discord_servers.insert(disc.uid, discord);
  }
  discord_servers
}

fn get_discord_players() -> Vec<DiscordPlayer> {
  let mut discord_players = vec![];
  for disc in &[&HEMO, &RAVENS, &NEEJIT] {
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

pub static DISCORDS: Lazy<Discords> = Lazy::new(|| get_discord_servers());
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
  fn discords() -> Result<(), String> { 
    let discords = get_discord_servers();
    if discords.is_empty() {
      Err("Can't get discord servers".into())
    } else {
      Ok(())
    }
  }
  #[test]
  fn players() -> Result<(), String> { 
    let discord_players = get_discord_players();
    if discord_players.is_empty() {
      Err("Can't get players".into())
    } else {
      Ok(())
    }
  }
}

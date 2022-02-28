use crate::types::team::{ Discords
                        , DiscordFields
                        , DiscordPlayer
                        , DiscordServer };

use serenity::model::id::ChannelId;

use once_cell::sync::Lazy;

fn grab_servers() -> Vec<DiscordServer> {
  match glob::glob("dhall/team/teams/*.dhall") {
    Ok(dfs) => dfs.filter_map(|d| d.ok())
                  .filter_map(|r| r.into_os_string()
                                   .into_string().ok())
                  .filter_map(|p| serde_dhall::from_file(p).parse().ok())
                  .collect::<Vec<DiscordServer>>(),
    Err(why) => {
      error!("Missing dhall/team/teams/: {why}"); vec![]
    }
  }
}

pub static SERVERS: Lazy<Vec<DiscordServer>> = Lazy::new(grab_servers);

fn get_discord_servers() -> Discords {
  let mut discord_servers: Discords = Discords::new();
  for disc in SERVERS.iter() {
    let discord = DiscordFields
                    { games:    disc.games
                    , games2:   disc.games2
                    , games4:   disc.games4
                    , streams:  disc.streams
                    , events:   disc.events.map(ChannelId)
                    , log:      disc.log.map(ChannelId)
                    };
    discord_servers.insert(disc.uid, discord);
  }
  discord_servers
}

fn get_discord_players() -> Vec<DiscordPlayer> {
  let mut discord_players = vec![];
  for disc in SERVERS.iter() {
    for player in disc.players.iter() {
      if let Some(existing) =
        discord_players.iter_mut()
                       .find(|dp: &&mut DiscordPlayer| (**dp).player.discord == player.discord) {
        existing.discords.push(disc.uid);
        if existing.player.battletag.is_empty() && !player.battletag.is_empty() {
          existing.player.battletag = player.battletag.clone();
        }
      } else {
        let discord_player = DiscordPlayer{ player: player.clone()
                                          , discords: vec![disc.uid] };
        discord_players.push(discord_player);
      }
    }
  }
  discord_players
}

pub static DISCORDS: Lazy<Discords> = Lazy::new(get_discord_servers);
pub static ALL: Lazy<Vec<DiscordPlayer>> = Lazy::new(get_discord_players);

fn get_only_battlenet_players() -> Vec<&'static DiscordPlayer> {
  ALL.iter().filter(|dp| !dp.player.battletag.is_empty())
            .collect::<Vec<&DiscordPlayer>>()
}

pub static PLAYERS: Lazy<Vec<&DiscordPlayer>> = Lazy::new(get_only_battlenet_players);

#[cfg(test)]
mod stuff_dhall_tests {
  use super::*;
  #[test]
  fn parsing_test() -> Result<(), String> {
    match glob::glob("dhall/team/teams/*.dhall") {
      Ok(dfs) => {
        let fnames = dfs.filter_map(|d| d.ok())
                        .filter_map(|r| r.into_os_string()
                                         .into_string().ok());
        for f in fnames {
          if let Err(dhall_error) = serde_dhall::from_file(&f).parse::<DiscordServer>() {
            return Err(format!("Failed to parese {f}, error: {dhall_error}"));
          }
        }
        Ok(())
      },
      Err(why) => {
        Err(format!("Missing dhall/team/teams/: {why}"))
      }
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

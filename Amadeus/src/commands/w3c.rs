use crate::{
  types::{
    serenity::ReqwestClient,
    w3c::*
  },
  common::{
    constants::W3C_API,
    msg::channel_message
  },
  steins::warcraft::
    utils::{ get_race, get_race2
           , get_league, get_map_short
           , get_league_png }
};

use serenity::{
  prelude::*,
  builder::CreateEmbed,
  model::channel::*,
  framework::standard::{
    Args, CommandResult
  , macros::command }
};

use serde_json::Value;
use comfy_table::*;

use std::{ time::Duration
         , collections::{ HashMap, BTreeMap }
         , sync::Arc
         , sync::atomic::Ordering::Relaxed
         , sync::atomic::AtomicU32 };
use async_std::fs;

use crate::common::constants::APM_PICS;
use plotters::prelude::*;

pub static CURRENT_SEASON: AtomicU32 = AtomicU32::new(11);
static ONGOING_PAGE_SIZE: usize = 15;

#[cfg(feature = "trackers")]
pub async fn update_current_season(ctx: &Context) {
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  if let Ok(res) = rqcl.get(&format!("{W3C_API}/ladder/seasons"))
                       .send()
                       .await {
    if let Ok(seasons) = res.json::<Vec<Season>>().await {
      let seasons_ids = seasons.iter().map(|s| s.id);
      if let Some(last_season) = seasons_ids.max() {
        CURRENT_SEASON.store(last_season, Relaxed);
      }
    }
  }
}

fn current_season() -> String {
  let atom = CURRENT_SEASON.load(Relaxed);
  format!("{atom}")
}

#[command]
#[description("shows ongoing matches on W3Champions")]
async fn ongoing(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  let url = format!("{W3C_API}/matches/ongoing?offset=0&gameMode=1");
  let res = rqcl.get(&url).send().await?;
  let going: Going = res.json().await?;
  if !going.matches.is_empty() {
    let footer = format!("Requested by {}", msg.author.name);
    let mut embeds = vec![];
    for (i, chunk) in going.matches.chunks(ONGOING_PAGE_SIZE).enumerate() {
      let mut embed = CreateEmbed::default();
      let mut description: String = String:: new();
      for m in chunk {
        if m.teams.len() > 1 && !m.teams[0].players.is_empty() && !m.teams[1].players.is_empty() {
          set! { g_map = get_map_short(&m.map)
               , race1 = get_race2(m.teams[0].players[0].race)
               , race2 = get_race2(m.teams[1].players[0].race) };
          let mstr = format!("({}) **{}** [{}] vs ({}) **{}** [{}] *{}*",
            race1, m.teams[0].players[0].name, m.teams[0].players[0].oldMmr
          , race2, m.teams[1].players[0].name, m.teams[1].players[0].oldMmr, g_map);
          description = format!("{}\n{}", mstr, description);
        }
      }
      embed.title(&format!("Ongoing matches, page {}", i + 1));
      embed.description(description);
      embed.thumbnail("https://i.pinimg.com/originals/b4/a0/40/b4a04082647a8505b3991cbaea7d2f86.png");
      embed.colour((180,40,200));
      embed.footer(|f| f.text(&footer));
      embeds.push(embed);
    }
    if !embeds.is_empty() {
      let mut page = 0;
      let mut bot_msg = msg.channel_id.send_message(ctx, |m| m.embed(|mut e| {
        e.0 = embeds[page].0.clone(); e
      })).await?;
      if embeds.len() > 1 {
        let left = ReactionType::Unicode(String::from("⬅️"));
        let right = ReactionType::Unicode(String::from("➡️"));
        let _ = bot_msg.react(ctx, left).await;
        let _ = bot_msg.react(ctx, right).await;
        loop {
          if let Some(reaction) =
            &bot_msg.await_reaction(ctx)
                    .author_id(msg.author.id.0)
                    .timeout(Duration::from_secs(120)).await {
            let emoji = &reaction.as_inner_ref().emoji;
            match emoji.as_data().as_str() {
              "⬅️" => { 
                if page != 0 {
                  page -= 1;
                }
              },
              "➡️" => { 
                if page != embeds.len() - 1 {
                  page += 1;
                }
              },
              _ => (),
            }
            bot_msg.edit(ctx, |m| m.embed(|mut e| {
              e.0 = embeds[page].0.clone(); e
            })).await?;
            let _ = reaction.as_inner_ref().delete(ctx).await;
          } else {
            let _ = bot_msg.delete_reactions(ctx).await;
            break;
          };
        }
      }
    }
  }
  Ok(())
}

async fn get_player(rqcl: &Arc<reqwest::Client>, target: &str, season: &str) -> anyhow::Result<Option<String>> {
  if target.contains('#') {
    Ok(Some(target.to_string()))
  }
  else {
    let search_uri =
      format!("{W3C_API}/ladder/search?gateWay=20&searchFor={target}&season={season}");
    let search: Vec<Search> = rqcl.get(&search_uri).send().await?.json::<Vec<Search>>().await?;
    if !search.is_empty() {
      // search for ToD will give toy Toddy at first, so we search for exact match
      for s in &search {
        for id in &s.player.playerIds {
          if target == id.name {
            return Ok(Some(id.battleTag.clone()));
          }
        }
      }
      // if there is no exact match return first search result
      Ok(Some(search[0].player.playerIds[0].battleTag.clone()))
    } else {
      Ok(None)
    }
  }
}

#[command]
#[aliases(статистика, statistics)]
#[description("display statistics on W3Champions")]
pub async fn stats(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let mut args_msg = args.message();
  if args_msg.is_empty() {
    args_msg = &msg.author.name;
  }
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  let season = current_season();
  if let Some(userx) = get_player(&rqcl, args_msg, &season).await? {
    let user = userx.replace('#',"%23");
    let game_mode_uri = format!("{W3C_API}/players/{user}/game-mode-stats?season={season}&gateWay=20");
    let game_mode_res = rqcl.get(&game_mode_uri).send().await?;
    let game_mode_stats: Vec<GMStats> =
      match game_mode_res.json::<Vec<GMStats>>().await {
        Ok(gms) => gms,
        Err(wha) => {
          let game_mode_res2 = rqcl.get(&game_mode_uri).send().await?;
          if let Ok(text_res) = game_mode_res2.text().await {
            error!("{wha} on {text_res}");
          }
          vec![]
        }
      };
    setm!{ league_info         = String::new()
         , ffa_info            = String::new()
         , rt_string           = String::new()
         , at_info             = String::new()
         , league_avi          = String::new() };
    let mut at_list: Vec<(u32, String)> = Vec::new();

    for gmstat in game_mode_stats {
      if gmstat.gameMode == 1 && league_info.is_empty() {
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
            String::from("Calibrating")
          } else {
            league_avi = get_league_png(lid);
            if lid > 1 {
              format!("*League*: **{league_str}** *Division:* **{}**", gmstat.division)
            } else {
              format!("*League*: **{league_str}**")
            }
          };
        league_info = format!("**Winrate**: **{winrate}%** **MMR**: __**{}**__\n{} *Rank*: **{}**",
          gmstat.mmr, &league_division, gmstat.rank);
      } else if gmstat.gameMode == 2 {
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{league_str}** *div:* **{}**", gmstat.division)
        } else {
          format!("**{league_str}**")
        };
        rt_string = format!("{} *games* {league_division} *Rank*: {} __**{winrate}%**__ *MMR*: __**{}**__",
          gmstat.games, gmstat.rank, gmstat.mmr);
      } else if gmstat.gameMode == 5 {
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{league_str}** *Division:* **{}**", gmstat.division)
        } else {
          format!("**{league_str}**")
        };
        ffa_info = format!("{league_division} *Rank*: **{}** *Winrate*: **{winrate}%** *MMR*: __**{}**__",
          gmstat.rank, gmstat.mmr);
      } else if gmstat.gameMode == 6 {
        let players = gmstat.playerIds;
        let mut player_str = String::new();
        for p in players {
          if p.battleTag != userx {
            player_str = p.name;
            break;
          }
        }
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{league_str}** *div:* **{}**", gmstat.division)
        } else {
          format!("**{league_str}**")
        };
        let strnfo = format!("__**{}**__ {} *games* {league_division} *Rank*: {} __**{winrate}%**__ *MMR*: __**{}**__",
          &player_str, gmstat.games, gmstat.rank, gmstat.mmr);
        at_list.push((gmstat.mmr, strnfo));
      }
    }
    if !at_list.is_empty() {
      at_list.sort_by(|(mmra,_), (mmrb, _) | mmra.cmp(mmrb));
      at_list.reverse();
      let map_of_sort: Vec<String> = at_list.into_iter().map(|(_, strx)| strx).take(5).collect();
      if !map_of_sort.is_empty() {
        at_info = map_of_sort.join("\n");
      }
    }

    let uri = format!("{W3C_API}/players/{user}/race-stats?season={season}&gateWay=20");
    let res = rqcl.get(&uri).send().await?;
    let stats: Vec<Stats> =
      match res.json::<Vec<Stats>>().await {
        Ok(sms) => sms,
        Err(wha) => {
          let sms_res_2 = rqcl.get(&uri).send().await?;
          if let Ok(text_res) = sms_res_2.text().await {
            error!("{:?} on {}", wha, text_res);
          }
          vec![]
        }
      };

    let mut stats_by_races: String = String::new();
    if !stats.is_empty() {

      let clan_uri = format!("{}/clans?battleTag={}", W3C_API, user);
      let name = &userx.split('#').collect::<Vec<&str>>()[0];
      let mut clanned = String::from(*name);
      if let Ok(clan_res) = rqcl.get(&clan_uri).send().await {
        if let Ok(clan_text_res) = clan_res.text().await {
          let clan_json_res = serde_json::from_str::<Value>(&clan_text_res);
          if let Ok(clan_json) = clan_json_res {
            if let Some(clan) = clan_json.pointer("/clanId") {
              if let Some(clan_str) = clan.as_str() {
                clanned = format!("[{}] {}", clan_str, name);
              }
            }
          }
        }
      }

      for stat in &stats {
        let race = get_race(stat.race);
        let winrate = (stat.winrate * 100.0).round();
        stats_by_races = format!("{stats_by_races}\n**{race}**\t: *wins*: {}, *loses*: {}, *winrate*: **{winrate}%**", stat.wins, stat.losses);
      }

      let max_games: Option<&Stats> = stats.iter().max_by_key(|s| s.games);
      let max_games_race =
        if let Some(max) = max_games {
          max.race
        } else { 0 };
      if league_avi.is_empty() {
        league_avi = match max_games_race {
            1 => "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/raceIcons/HUMAN.png",
            2 => "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/raceIcons/ORC.png",
            4 => "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/raceIcons/NIGHT_ELF.png",
            8 => "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/raceIcons/UNDEAD.png",
            _ => "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/raceIcons/RANDOM.png"
          }.to_string();
      }
      let main_race_colors = match max_games_race {
          1 => (0, 0, 222),
          2 => (222, 0, 0),
          4 => (0, 222, 0),
          8 => (155, 0, 143),
          _ => (50, 120, 150)
        };

      let mut description = format!("[{}] {}\n", &userx, &league_info);

      let uri2 = format!("{W3C_API}/player-stats/{user}/race-on-map-versus-race?season={season}");
      let res2 = rqcl.get(&uri2).send().await?;
      let stats2_mb: Option<Stats2> =
        match res2.json::<Stats2>().await {
          Ok(sms2) => Some(sms2),
          Err(wha2) => {
            let sms2_res_2 = rqcl.get(&uri2).send().await?;
            if let Ok(text_res) = sms2_res_2.text().await {
              error!("{wha2} on {text_res}");
            }
            None
          }
        };

      if let Some(stats2) = stats2_mb {
        let mut table = Table::new();

        table.set_content_arrangement(ContentArrangement::Dynamic)
             .set_width(40)
             .set_header(vec!["Map", "vs HU", "vs O", "vs NE", "vs UD"]);

        if let Some(s24) = stats2.raceWinsOnMapByPatch.get("All") {
          for s3 in s24 {
            if !s3.winLossesOnMap.is_empty() &&
                s3.race == 16 {
              for s4 in &s3.winLossesOnMap {
                let text = get_map_short(&s4.map);
                let mut scores: HashMap<u32, String> = HashMap::new();
                for s5 in &s4.winLosses {
                  let vs_winrate = (s5.winrate * 100.0).round();
                  let text = format!("{vs_winrate}%");
                  scores.insert(s5.race, text);
                }
                table.add_row(vec![
                  Cell::new(text).set_alignment(CellAlignment::Left),
                  Cell::new(scores.get(&1).unwrap_or( &String::from("-") ))
                    .set_alignment(CellAlignment::Center),
                  Cell::new(scores.get(&2).unwrap_or( &String::from("-") ))
                    .set_alignment(CellAlignment::Center),
                  Cell::new(scores.get(&4).unwrap_or( &String::from("-") ))
                    .set_alignment(CellAlignment::Center),
                  Cell::new(scores.get(&8).unwrap_or( &String::from("-") ))
                    .set_alignment(CellAlignment::Center)
                ]);
              }
            }
          }
        }
        description = format!("{description}```\n{table}\n```");
      }

      let footer = if !msg.author.bot {
          format!("Requested by {}", msg.author.name)
        } else {
          String::from("Requested from /")
        };

      let mut additional_info = vec![("Stats by races", &stats_by_races, false)];
      if !rt_string.is_empty() {
        additional_info.push(("RT 2x2", &rt_string, false));
      }
      if !at_info.is_empty() {
        additional_info.push(("AT 2x2", &at_info, false));
      }
      if !ffa_info.is_empty() {
        additional_info.push(("FFA", &ffa_info, false));
      }

      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
          .title(&clanned)
          .description(description)
          .url(&format!("https://www.w3champions.com/player/{}", user))
          .thumbnail(&league_avi)
          .fields(additional_info)
          .colour(main_race_colors)
          .footer(|f| f.text(footer)))).await {
        error!("Error sending stats message: {why}");
      }
    } else {
      let resp = format!("User {args_msg} not found");
      channel_message(ctx, msg, &resp).await;
    }
  } else {
    let resp = format!("Search on {args_msg} found no users");
    channel_message(ctx, msg, &resp).await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

#[command]
#[min_args(2)]
#[description("Generates ideal veto based on W3C statistics")]
async fn veto(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let args_msg = args.single::<String>()?;
  let race_vs = args.single::<String>()?;

  let start_typing = ctx.http.start_typing(msg.channel_id.0);

  let mut seasons = 2;
  let season_str = current_season();
  let season = season_str.parse::<u32>()?;

  if let Ok(opt) = args.single::<String>() {
    let lower = opt.to_lowercase();
    if lower == "all" {
      seasons = season - 1;
    } else if lower == "last" {
      seasons = 1;
    }
  }
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };

  if let Some(userx) = get_player(&rqcl, &args_msg, &season_str).await? {
    let user = userx.replace('#',"%23");

    let uri2 = format!("{W3C_API}/player-stats/{user}/race-on-map-versus-race?season={season}");
    let res2 = rqcl.get(&uri2).send().await?;
    let stats2_mb: Option<Stats2> =
      match res2.json::<Stats2>().await {
        Ok(sms2) => Some(sms2),
        Err(wha2) => {
          let sms2_res_2 = rqcl.get(&uri2).send().await?;
          if let Ok(text_res) = sms2_res_2.text().await {
            error!("{wha2} on {text_res}");
          }
          None
        }
      };

    if let Some(stats2) = stats2_mb {
      let race_vs_lower = race_vs.to_lowercase();
      let race_vs_num: u32 =
        if race_vs_lower.starts_with('h') {
          1
        } else if race_vs_lower.starts_with('o') {
          2
        } else if race_vs_lower.starts_with('n')
              || race_vs_lower.starts_with('e') {
          4
        } else if race_vs_lower.starts_with('u') {
          8
        } else {
          0
        };

      if race_vs_num == 0 {
        channel_message(ctx, msg, "Can't parse that race").await;
        if let Ok(typing) = start_typing {
          typing.stop();
        }
        return Ok(());
      }

      let mut winrate_maps = vec![];
      let mut process_stats2 = |stats2: Stats2| {
        if let Some(s24) = stats2.raceWinsOnMapByPatch.get("All") {
          for s3 in s24 {
            if !s3.winLossesOnMap.is_empty() &&
                s3.race == 16 {
              for s4 in &s3.winLossesOnMap {
                let text_map = get_map_short(&s4.map);
                for s5 in &s4.winLosses {
                  if s5.race == race_vs_num && text_map != "All" {
                    if let Some(fwm) =
                      winrate_maps.iter_mut().find(|(_, m, _, _)|
                      m == &text_map
                    ) {
                      let (_, _, ww, ll) = fwm;
                      let aw = *ww + s5.wins;
                      let al = *ll + s5.losses;
                      let wr: f64 =
                        if al + aw > 0 {
                          (aw as f64/(al as f64+aw as f64) * 100.0).round()
                        } else { 0.0 };
                      *fwm = (wr, text_map.clone(), aw, al);
                    } else if !(s5.wins == 0 && s5.losses == 0) {
                      let vs_winrate = (s5.winrate * 100.0).round();
                      winrate_maps.push(( vs_winrate, text_map.clone()
                                        , s5.wins, s5.losses ));
                    }
                  }
                }
              }
            }
          }
        }
      };

      process_stats2(stats2);

      for sx in 0..seasons {
        let previous_season = season - sx;
        let uri3 = format!("{W3C_API}/player-stats/{user}/race-on-map-versus-race?season={previous_season}");
        if let Ok(res3) = rqcl.get(&uri3).send().await {
          if let Ok(stats3) = res3.json::<Stats2>().await {
            process_stats2(stats3);
          }
        }
      }

      winrate_maps.sort_by(|(a,_,_,_), (b,_,_,_)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Less));

      let mut out = String::new();
      for (w, m, ww, ll) in winrate_maps {
        out = format!("{out}**{m}**\t\t{w}% **{ww}**W - **{ll}**L\n");
      }

      let footer = format!("Requested by {}", msg.author.name);
      if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
          .embed(|e| e
          .title(&format!("{} vs {}", &userx, &race_vs))
          .description(out)
          .url(&format!("https://www.w3champions.com/player/{user}/statistics"))
          .footer(|f| f.text(footer)))).await {
        error!("Error sending veto message: {why}");
      }
    }

  } else {
    channel_message(ctx, msg, "Search found no users with that nickname").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

#[command]
#[min_args(2)]
#[aliases(versus)]
#[description("Show W3C statistics for two players")]
async fn vs(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let p1 = args.single::<String>()?;
  let p2 = args.single::<String>()?;

  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let mut seasons = 2;
  let season_str = current_season();
  let season = season_str.parse::<u32>()?;

  if let Ok(opt) = args.single::<String>() {
    let lower = opt.to_lowercase();
    if lower == "all" {
      seasons = season - 1;
    } else if lower == "last" {
      seasons = 1;
    }
  }

  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };

  if let Some(userx1) = get_player(&rqcl, &p1, &season_str).await? {
    if let Some(userx2) = get_player(&rqcl, &p2, &season_str).await? {
      let name1 = &userx1.split('#').collect::<Vec<&str>>()[0];
      let name2 = &userx2.split('#').collect::<Vec<&str>>()[0];

      let user1 = userx1.replace('#',"%23");
      let user2 = userx2.replace('#',"%23");

      let mut match_strings = vec![];
      let mut wins = 0;
      let mut loses = 0;
      for sx in 0..seasons {
        let previous_season = season - sx;
        let vs_uri = format!("{}/matches/search?playerId={}&gateway=20&offset=0&opponentId={}&season={}",
                                W3C_API, user1, user2, previous_season);

        debug!("VS: {vs_uri}");
        let ress = rqcl.get(&vs_uri).send().await?;
        let rest: Going = ress.json().await?;
        for m in rest.matches.iter() {
          // for now only solo matches
          if m.gameMode == 1 {
            let map_name = get_map_short(&m.map);
            let flo_info =
              if m.serverInfo.provider == Some("BNET".to_string()) {
                String::from("BNET")
              } else if let Some(si) = &m.serverInfo.name {
                si.clone()
              } else {
                "BNET".to_string()
              };
            let mut p1s = String::new();
            let mut p2s = String::new();
            let mut winner = false;
            for t in m.teams.iter() {
              for p in t.players.iter() {
                let race = get_race2(p.race);
                let mut if_ping = String::new();
                for psi in m.serverInfo.playerServerInfos.iter() {
                  if psi.battleTag == p.battleTag {
                    if_ping = format!(" {}ms", psi.averagePing);
                  }
                }
                if p.battleTag == userx1 {
                  if t.won {
                    winner = true;
                    wins += 1;
                  } else {
                    loses += 1;
                  }
                  p1s = format!("{name1} ({race}) {if_ping}");
                } else {
                  p2s = format!("{name2} ({race}) {if_ping}");
                }
              }
            }
            let match_string = 
              if winner {
                format!(
                  "• [{}, {}] **{}** > {} <https://www.w3champions.com/match/{}>",
                  map_name, flo_info, p1s, p2s, m.id )
                } else {
                  format!(
                    "• [{}, {}] {} < **{}** <https://www.w3champions.com/match/{}>",
                    map_name, flo_info, p1s, p2s, m.id )
                };
            match_strings.push(match_string);
          }
        }
      }

      if !match_strings.is_empty() {
        let footer = format!("Requested by {}", msg.author.name);
        if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
            .embed(|e| e
            .title(&format!("{} {} : {} {}", &name1, wins, loses, &name2))
            .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
            .description(match_strings.join("\n"))
            .footer(|f| f.text(footer)))).await {
          error!("Error sending veto message: {why}");
        }
      } else {
        channel_message(ctx, msg, "No games for those players in selected seasons").await;
      }
    } else {
      channel_message(ctx, msg, &format!("Can't find {p2}")).await;
    }
  } else {
    channel_message(ctx, msg, &format!("Can't find {p1}")).await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {why}");
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

pub async fn generate_popularhours(ctx: &Context) -> anyhow::Result<Option<String>> {
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  let mut popular_games_image: Option<String> = None;
  if let Ok(res3) = rqcl.get(format!("{W3C_API}/w3c-stats/play-hours")).send().await {
    if let Ok(ph_modes) = res3.json::<Vec<PopularHours>>().await {
      info!("got popular hours structure");
      for ph in ph_modes {
        if ph.gameMode == 2 {
          let max_games = ph.playTimePerHour.iter().fold(0, |a, b| b.games.max(a));
          let fname_popular_hours = format!("popular_hours_{}.png", ph.day);
          { // because of Rc < > in BitMapBackend I need own scope here
            let root_area = BitMapBackend::new(&fname_popular_hours, (1024, 384)).into_drawing_area();
            root_area.fill(&RGBColor(47, 49, 54))?;
            let mut cc = ChartBuilder::on(&root_area)
              .margin(5)
              .set_all_label_area_size(50)
              .build_cartesian_2d(0.0..24_f64, 0.0..max_games as f64)?;
            cc.configure_mesh()
              .label_style(("monospace", 16).into_font().color(&RGBColor(150, 150, 150)))
              .y_labels(10)
              .axis_style(&RGBColor(80, 80, 80))
              .draw()?;
            let color = RGBColor(180, 120, 255);
            let style: ShapeStyle = ShapeStyle::from(color);
            let plx = ph.playTimePerHour.iter().map(|p| {
              (p.hours as f64 + (p.minutes as f64 / 64f64), p.games as f64)
            }).collect::<Vec<(f64, f64)>>();
            cc.draw_series(LineSeries::new(plx, style.clone()))?
              .label("2x2")
              .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style.clone()));
            cc.configure_series_labels()
              .position(SeriesLabelPosition::LowerRight)
              .border_style(&BLACK)
              .label_font(("monospace", 19).into_font().color(&RGBColor(200, 200, 200)))
              .draw()?;
          }
          match APM_PICS.send_message(&ctx, |m|
            m.add_file(AttachmentType::Path(std::path::Path::new(&fname_popular_hours)))).await {
            Ok(msg) => {
              if !msg.attachments.is_empty() {
                let img_attachment = &msg.attachments[0];
                popular_games_image = Some(img_attachment.url.clone());
              }
            },
            Err(why) => {
              error!("Failed to download and post stream img {why}");
            }
          };
        }
      }
    } else {
      error!("failed to parse");
    }
  } else {
    error!("no rsponse from w3c");
  }
  Ok(popular_games_image)
}

#[command]
#[description("Show popular hours on W3C")]
async fn popularhours(ctx: &Context, msg: &Message) -> CommandResult {
  let popular_games_image = generate_popularhours(ctx).await;
  if let Some(img) = popular_games_image? {
    let footer = format!("Requested by {}", msg.author.name);
    msg.channel_id.send_message(ctx, |m| m.content("")
    .embed(|e|
      e.color((40, 20, 200))
       .title("2x2 popular hours")
       .image(img)
       .footer(|f| f.text(&footer))
    )).await?;
    if let Err(why) = msg.delete(&ctx).await {
      error!("Error deleting original command, {why}");
    }
  }
  Ok(())
}

const MMM_FNAME: &str = "mmm.yml";

fn avg(numbers: &[u32]) -> u32 {
  let avg = numbers.iter().sum::<u32>() as f32 / numbers.len() as f32;
  avg.round() as u32
}

pub async fn get_mmm(ctx: &Context) -> anyhow::Result<(u32, u32, u32)> {
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  let res = rqcl.get("https://matchmaking-service.w3champions.com/queue/snapshots").send().await?;
  let parsed = res.json::<Vec<QueueSnapshot>>().await?;
  info!("parsed mmm");
  setm!{ qtime1 = vec![]
       , qtime2 = vec![]
       , qtime4 = vec![] };
  if !std::path::Path::new(MMM_FNAME).exists() {
    let mut data: BTreeMap<String, PlayerData> = BTreeMap::new();
    for qs in parsed {
      for s in qs.snapshot {
        if qs.gameMode == 1 {
          qtime1.push( s.queueTime );
        } else if qs.gameMode == 2 {
          qtime2.push( s.queueTime );
        } else if qs.gameMode == 4 {
          qtime4.push( s.queueTime );
        }
        for p in s.playerData {
          let p_clone = p.clone();
          data.insert(p.battleTag, p_clone);
        }
      }
    }
    let yml = serde_yaml::to_string(&data)?;
    fs::write(MMM_FNAME, yml).await?;
  } else {
    let contents = fs::read_to_string(MMM_FNAME).await?;
    let mut data: BTreeMap<String, PlayerData> = serde_yaml::from_str(&contents)?;
    for qs in parsed {
      for s in qs.snapshot {
        for p in s.playerData {
          let p_clone = p.clone();
          if let Some(d) = data.get_mut(&p.battleTag) {
            // override if exists
            *d = p_clone;
          } else {
            let p_clone = p.clone();
            data.insert(p.battleTag, p_clone);
          }
        }
      }
    }
    let yml = serde_yaml::to_string(&data)?;
    fs::write(MMM_FNAME, yml).await?;
  }
  Ok(( avg(&qtime1)
     , avg(&qtime2)
     , avg(&qtime4) ))
}

#[command]
#[description("Get")]
#[owners_only]
async fn mmm(ctx: &Context, msg: &Message) -> CommandResult {
  let _qtime = get_mmm(ctx).await?;
  let footer = format!("Requested by {}", msg.author.name);
  msg.channel_id.send_message(ctx, |m| m.content("")
  .embed(|e|
    e.color((40, 20, 200))
      .title("Ok")
      .footer(|f| f.text(&footer))
  )).await?;
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  Ok(())
}

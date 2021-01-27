use crate::{
  types::{
    common::ReqwestClient,
    w3c::*
  },
  common::{
    msg::{ channel_message }
  },
  steins::cyber::{
    utils::{ get_race, get_race2
           , get_league, get_map, get_league_png }
  }
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
         , collections::HashMap
         , sync::Arc
         , sync::atomic::Ordering::Relaxed
         , sync::atomic::AtomicU32 };

pub static CURRENT_SEASON: AtomicU32 = AtomicU32::new(5);
static ONGOING_PAGE_SIZE: usize = 15;

pub async fn update_current_season(ctx: &Context) {
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  if let Ok(res) = rqcl.get("https://statistic-service.w3champions.com/api/ladder/seasons")
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
  format!("{}", atom)
}

#[command]
#[description("shows ongoing matches on W3Champions")]
async fn ongoing(ctx: &Context, msg: &Message) -> CommandResult {
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };
  let url = "https://statistic-service.w3champions.com/api/matches/ongoing?offset=0&gameMode=1";
  let res = rqcl.get(url).send().await?;
  let going : Going = res.json().await?;
  if !going.matches.is_empty() {
    let footer = format!("Requested by {}", msg.author.name);
    let mut embeds = vec![];
    for (i, chunk) in going.matches.chunks(ONGOING_PAGE_SIZE).enumerate() {
      let mut embed = CreateEmbed::default();
      let mut description : String = String:: new();
      for m in chunk {
        if m.teams.len() > 1 && !m.teams[0].players.is_empty() && !m.teams[1].players.is_empty() {
          set! { g_map = get_map(&m.map)
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

async fn get_player(rqcl: &Arc<reqwest::Client>, target: &str, season: &str) -> eyre::Result<Option<String>> {
  if target.contains('#') {
    Ok(Some(target.to_string()))
  }
  else {
    let search_uri =
      format!("https://statistic-service.w3champions.com/api/ladder/search?gateWay=20&searchFor={}&season={}"
             , target, season);
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
#[aliases(статистика)]
#[description("display statistics on W3Champions")]
async fn stats(ctx: &Context, msg: &Message, args : Args) -> CommandResult {
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
    let user = userx.replace("#","%23");
    let game_mode_uri = format!("https://statistic-service.w3champions.com/api/players/{}/game-mode-stats?season={}&gateWay=20", user, season);
    let game_mode_res = rqcl.get(&game_mode_uri).send().await?;
    let game_mode_stats : Vec<GMStats> =
      match game_mode_res.json::<Vec<GMStats>>().await {
        Ok(gms) => gms,
        Err(wha) => {
          let game_mode_res2 = rqcl.get(&game_mode_uri).send().await?;
          if let Ok(text_res) = game_mode_res2.text().await {
            error!("{:?} on {}", wha, text_res);
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
              format!("*League*: **{}** *Division:* **{}**", league_str, gmstat.division)
            } else {
              format!("*League*: **{}**", league_str)
            }
          };
        let progr = if gmstat.rankingPointsProgress.mmr > 0 {
            format!("+{}", gmstat.rankingPointsProgress.mmr)
          } else {
            gmstat.rankingPointsProgress.mmr.to_string()
          };
        league_info = format!("**Winrate**: **{}%** **MMR**: __**{}**__ (*{}*)\n{} *Rank*: **{}**",
          winrate, gmstat.mmr, progr, &league_division, gmstat.rank);
      } else if gmstat.gameMode == 2 {
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{}** *div:* **{}**", league_str, gmstat.division)
        } else {
          format!("**{}**", league_str)
        };
        rt_string = format!("{} *games* {} *Rank*: {} __**{}%**__ *MMR*: __**{}**__",
          gmstat.games, league_division, gmstat.rank, winrate, gmstat.mmr);
      } else if gmstat.gameMode == 5 {
        set!{ lid         = gmstat.leagueOrder
            , league_str  = get_league(lid)
            , winrate     = (gmstat.winrate * 100.0).round() };
        let league_division = if gmstat.games < 5 {
          String::from("Calibrating")
        } else if lid > 1 {
          format!("**{}** *Division:* **{}**", league_str, gmstat.division)
        } else {
          format!("**{}**", league_str)
        };
        ffa_info = format!("{} *Rank*: **{}** *Winrate*: **{}%** *MMR*: __**{}**__",
          league_division, gmstat.rank, winrate, gmstat.mmr);
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
          format!("**{}** *div:* **{}**", league_str, gmstat.division)
        } else {
          format!("**{}**", league_str)
        };
        let strnfo = format!("__**{}**__ {} *games* {} *Rank*: {} __**{}%**__ *MMR*: __**{}**__",
          &player_str, gmstat.games, league_division, gmstat.rank, winrate, gmstat.mmr);
        at_list.push((gmstat.mmr, strnfo));
      }
    }
    if !at_list.is_empty() {
      at_list.sort_by(|(mmra,_), (mmrb, _) | mmra.cmp(mmrb));
      at_list.reverse();
      let map_of_sort : Vec<String> = at_list.into_iter().map(|(_, strx)| strx).take(5).collect();
      if !map_of_sort.is_empty() {
        at_info = map_of_sort.join("\n");
      }
    }

    let uri = format!("https://statistic-service.w3champions.com/api/players/{}/race-stats?season={}&gateWay=20", user, season);
    let res = rqcl.get(&uri).send().await?;
    let stats : Vec<Stats> = res.json().await?;

    let mut stats_by_races : String = String::new();
    if !stats.is_empty() {

      let clan_uri = format!("https://statistic-service.w3champions.com/api/clans?battleTag={}", user);
      let name = &userx.split('#').collect::<Vec<&str>>()[0];
      let mut clanned = String::from(*name);
      if let Ok(clan_res) = rqcl.get(&clan_uri).send().await {
        if let Ok(clan_text_res) = clan_res.text().await {
          let clan_json_res = serde_json::from_str(&clan_text_res);
          if clan_json_res.is_ok() {
            let clan_json : Value = clan_json_res.unwrap();
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
        stats_by_races = format!("{}\n**{}**\t : *wins*: {}, *loses*: {}, *winrate*: **{}%**", stats_by_races, race, stat.wins, stat.losses, winrate);
      }

      let max_games : Option<&Stats> = stats.iter().max_by_key(|s| s.games);
      let max_games_race = if max_games.is_some() { max_games.unwrap().race } else { 0 };
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

      let uri2 = format!("https://statistic-service.w3champions.com/api/player-stats/{}/race-on-map-versus-race?season={}", user, season);
      let res2 = rqcl.get(&uri2).send().await?;
      let stats2 : Stats2 = res2.json().await?;

      let mut table = Table::new();

      table.set_content_arrangement(ContentArrangement::Dynamic)
           .set_table_width(40)
           .set_header(vec!["Map", "vs HU", "vs O", "vs NE", "vs UD"]);

      if let Some(s24) = stats2.raceWinsOnMapByPatch.get("All") {
        for s3 in s24 {
          if !s3.winLossesOnMap.is_empty() &&
              s3.race == 16 {
            for s4 in &s3.winLossesOnMap {
              let text = get_map(&s4.map);
              let mut scores : HashMap<u32, String> = HashMap::new();
              for s5 in &s4.winLosses {
                let vs_winrate = (s5.winrate * 100.0).round();
                let text = format!("{}%", vs_winrate);
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

      description = format!("{}```\n{}\n```", description, table);
      let footer = format!("Requested by {}", msg.author.name);

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
        error!("Error sending stats message: {:?}", why);
      }
    } else {
      let resp = format!("User {} not found", args_msg);
      channel_message(&ctx, &msg, &resp).await;
    }
  } else {
    let resp = format!("Search on {} found no users", args_msg);
    channel_message(&ctx, &msg, &resp).await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

#[command]
#[min_args(2)]
#[description("Generates ideal veto based on W3C statistics")]
async fn veto(ctx: &Context, msg: &Message, mut args : Args) -> CommandResult {
  let start_typing = ctx.http.start_typing(msg.channel_id.0);
  let args_msg = args.single::<String>()?;
  let race_vs = args.single::<String>()?;
  let mut seasons = 2;
  let season = current_season();
  if let Ok(opt) = args.single::<String>() {
    let lower = opt.to_lowercase();
    if lower == "all" {
      seasons = season.parse::<u32>().unwrap() - 1;
    } else if lower == "last" {
      seasons = 1;
    }
  }
  let rqcl = {
    set!{ data = ctx.data.read().await
        , rqcl = data.get::<ReqwestClient>().unwrap() };
    rqcl.clone()
  };

  let userx = if args_msg.contains('#') { args_msg }
    else {
      let search_uri = format!("https://statistic-service.w3champions.com/api/ladder/search?searchFor={}&season={}&gateWay=20", args_msg, season);
      let ress = rqcl.get(&search_uri).send().await?;
      let search : Vec<Search> = ress.json().await?;
      if !search.is_empty() {
        if !search[0].player.playerIds.is_empty() {
          search[0].player.playerIds[0].battleTag.clone()
        } else { String::new() }
      } else { String::new() }
    };
  if !userx.is_empty() {
    let user = userx.replace("#","%23");

    let uri2 = format!("https://statistic-service.w3champions.com/api/player-stats/{}/race-on-map-versus-race?season={}", user, season);
    let res2 = rqcl.get(&uri2).send().await?;
    let stats2 : Stats2 = res2.json().await?;

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
      channel_message(&ctx, &msg, "Can't parse that race").await;
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
              let text_map = get_map(&s4.map);
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

    for sx in 1..seasons {
      let previous_season = season.parse::<u32>().unwrap() - sx;
      let uri3 = format!("https://statistic-service.w3champions.com/api/player-stats/{}/race-on-map-versus-race?season={}", user, previous_season);
      if let Ok(res3) = rqcl.get(&uri3).send().await {
        if let Ok(stats3) = res3.json::<Stats2>().await {
          process_stats2(stats3);
        }
      }
    }

    winrate_maps.sort_by(|(a,_,_,_), (b,_,_,_)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Less));

    let mut out = String::new();
    for (w, m, ww, ll) in winrate_maps {
      out = format!("{}**{}**\t\t{}% **{}**W - **{}**L\n", out, m, w, ww, ll);
    }

    let footer = format!("Requested by {}", msg.author.name);
    if let Err(why) = msg.channel_id.send_message(&ctx, |m| m
        .embed(|e| e
        .title(&format!("{} vs {}", &userx, &race_vs))
        .description(out)
        .url(&format!("https://www.w3champions.com/player/{}/statistics", user))
        .footer(|f| f.text(footer)))).await {
      error!("Error sending veto message: {:?}", why);
    }

  } else {
    channel_message(&ctx, &msg, "Search found no users with that nickname").await;
  }
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  if let Ok(typing) = start_typing {
    typing.stop();
  }
  Ok(())
}

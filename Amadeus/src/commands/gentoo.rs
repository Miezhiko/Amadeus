use crate::{
  common::{
    msg::channel_message,
    giveaway
  },
  types::{
    gentoo::*,
    serenity::ReqwestClient
  }
};

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateEmbed, CreateEmbedFooter },
  model::{
    channel::*,
    id::UserId
  },
  framework::standard::{
    CommandResult, Args,
    macros::command
  }
};

use std::collections::HashSet;

use rand::{
  distributions::{
    WeightedIndex,
    Distribution
  }
};

use chrono::DateTime;

use tokio::task;

use nipper::Document;

#[command]
#[description("Find bugzilla bug by number")]
#[min_args(1)]
async fn bug(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let number = args.single::<u64>()?;
  let reqwest_client = {
    set!{ data            = ctx.data.read().await
        , reqwest_client  = data.get::<ReqwestClient>().unwrap() };
    reqwest_client.clone()
  };
  let res = reqwest_client.get(&format!("https://bugs.gentoo.org/rest/bug?id={number}")).send().await?;
  let bugs: Bugs = res.json().await?;
  if let Some(bug) = bugs.bugs.first() {
    let footer = format!("Requested by {}", msg.author.name);
    let mut e = CreateEmbed::new()
      .title(&bug.summary)
      .url(format!("https://bugs.gentoo.org/{number}"))
      .color((255, 0, 0))
      .footer(CreateEmbedFooter::new(footer));
    if !bug.assigned_to.is_empty() {
      e = e.field("assigned", &bug.assigned_to, true);
    }
    if !bug.creation_time.is_empty() {
      if let Ok(dt) = DateTime::parse_from_rfc3339(&bug.creation_time) {
        e = e.timestamp(dt);
      }
    }
    if !bug.creator.is_empty() {
      e = e.field("creator", &bug.creator, true);
    }
    if !bug.priority.is_empty() {
      e = e.field("priority", &bug.priority, true);
    }
    if !bug.severity.is_empty() {
      e = e.field("severity", &bug.severity, true);
    }
    if !bug.product.is_empty() {
      e = e.field("product", &bug.product, true);
    }
    if !bug.resolution.is_empty() {
      e = e.field("resolution", &bug.resolution, true);
    }
    if !bug.status.is_empty() {
      e = e.field("status", &bug.status, true);
    }
    if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
      .embed(e)
    ).await {
      msg.channel_id.say(ctx, &format!("Error: {why}")).await?;
    };
  } else {
    channel_message(ctx, msg, &format!("no bugs found with number: {number}")).await;
  }
  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[description("Find package atom in Gentoo overlays")]
#[min_args(1)]
#[aliases(overlays)]
async fn zugaina(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let search = args.message().to_string();
  let reqwest_client = {
    set!{ data            = ctx.data.read().await
        , reqwest_client  = data.get::<ReqwestClient>().unwrap() };
    reqwest_client.clone()
  };

  let url = format!("http://gpo.zugaina.org/Search?search={}", &search);
  let resp = reqwest_client.get(&url)
                           .send()
                           .await?
                           .text()
                           .await?;

  let resp_clone = resp.clone();
  let mut pages = task::spawn_blocking(move || -> usize {
    let document = Document::from(&resp_clone);
    let pages = document.nip("div[id=\"contentInner\"] > div[class=\"pager\"] > a[href]");
    // div 2 because there are pages on top and bottom and they look same
    pages.size() / 2
  }).await?;

  let mut top_level = vec![];
  let mut result_vec = vec![];
  let search_local = search.clone();
  result_vec.push(
    task::spawn_blocking(move || -> Vec<(String, String, String)> {
      let document = Document::from(&resp);
      document.nip("a > div").iter().take(5).flat_map(|element| {
        let text = element.text();
        let (atom, description)   = text.split_once(' ')?;
        let (_category, pkgname)  = atom.split_once('/')?;
        if pkgname.contains(&search_local) {
          Some((
            atom.to_string(),
            format!("http://gpo.zugaina.org/{atom}"),
            format!("**[{atom}](http://gpo.zugaina.org/{atom})** {description}")
          ))
        } else { None }
      }).collect::<Vec<(String, String, String)>>()
    }).await?
  );
  if pages > 0 {
    // it's hard to get all the pages from start so let take like first 30 pages
    // we will stop processing once we will get no results on the page
    if pages > 7 {
      pages = 30;
    }
    for p in 0..pages {
      let page = p + 2;
      let urlx = format!("https://gpo.zugaina.org/Search?search={}&use=&page={page}", &search);
      let respx = reqwest_client.get(&urlx)
                                .send()
                                .await?
                                .text()
                                .await?;
      let search_local = search.clone();
      let page_results =
        task::spawn_blocking(move || -> Vec<(String, String, String)> {
          let document = Document::from(&respx);
          document.nip("a > div").iter().take(5).flat_map(|element| {
            let text = element.text();
            let (atom, description)   = text.split_once(' ')?;
            let (_category, pkgname)  = atom.split_once('/')?;
            if pkgname.contains(&search_local) {
              Some((
                atom.to_string(),
                format!("http://gpo.zugaina.org/{atom}"),
                format!("**[{atom}](http://gpo.zugaina.org/{atom})** {description}")
              ))
            } else { None }
          }).collect::<Vec<(String, String, String)>>()
        }).await?;
      if page_results.is_empty() {
        break;
      }
      result_vec.push(page_results);
    }
  };

  for tlv in &mut result_vec {
    top_level.append(tlv);
  }

  let mut parse_result = vec![];
  for (atom, pkg_url, desc) in top_level {
    let pkg_resp = reqwest_client.get(&pkg_url).send().await?.text().await?;
    let pkg_level = task::spawn_blocking(move || -> Vec<String> {
      let document = Document::from(&pkg_resp);
      document.nip("div > li").iter().take(5).flat_map(|element| {
        let text  = element.text();
        let split = text.split(|c| c == ' ' || c == '\n' || c == '\t')
                        .filter(|&x| !x.is_empty())
                        .collect::<Vec<&str>>();
        if split.is_empty() {
          None
        } else {
          let first = split.first()?;
          let last  = split.last()?;
          Some(format!(" • **{first}** from [{last}](https://data.gpo.zugaina.org/{last}/{atom})"))
        }
      }).collect::<Vec<String>>()
    }).await?;
    let pkg_level_str = pkg_level.join("\n");
    parse_result.push( format!("{desc}\n{pkg_level_str}") );
  }

  let parse_result_str = parse_result.join("\n\n");
  let footer = format!("Requested by {}", msg.author.name);
  if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(CreateEmbed::new()
      .title(&search)
      .url(&url)
      .description(parse_result_str)
      .footer(CreateEmbedFooter::new(footer))
    )
  ).await {
    msg.channel_id.say(ctx, &format!("Error: {why}")).await?;
  };

  Ok(())
}

#[command]
#[description("Find Gentoo Wiki article")]
#[min_args(1)]
async fn wiki(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let search_text = args.single::<String>()?;
  let maybe_second = args.single::<String>();
  let reqwest_client = {
    set!{ data            = ctx.data.read().await
        , reqwest_client  = data.get::<ReqwestClient>().unwrap() };
    reqwest_client.clone()
  };
  let res = reqwest_client.get(&format!("https://wiki.gentoo.org/api.php?action=opensearch&search={search_text}")).send().await?;
  let (search_request, texts, _, links): Wiki = res.json().await?;

  if links.is_empty() {
    channel_message(ctx, msg, &format!("nothing found for: {search_text}")).await;
    return Ok(());
  }

  let footer = format!("Requested by {}", msg.author.name);
  let mut e = CreateEmbed::new()
                .title(&search_request)
                .url( links.first().unwrap_or(&"https://wiki.gentoo.org".to_string()) )
                .color((240, 0, 170))
                .footer(CreateEmbedFooter::new(footer));

  let mut filtered_result = false;
  if let Ok(other) = maybe_second {
    let other_lowered = other.to_lowercase();
    for (i, link) in links.iter().enumerate() {
      if let Some(title) = texts.get(i) {
        let title_lowered = title.to_lowercase();
        if title_lowered.contains(&other_lowered) {
          e = e.field(title, link, false);
          if !filtered_result { filtered_result = true; }
        }
      }
    }
  }

  if !filtered_result {
    for (i, link) in links.iter().enumerate() {
      if let Some(title) = texts.get(i) {
        e = e.field(title, link, false);
      }
    }
  }

  if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(e)
  ).await {
    msg.channel_id.say(ctx, &format!("Error: {why}")).await?;
  };

  if let Err(why) = msg.delete(ctx).await {
    error!("Error deleting original command {why}");
  }
  Ok(())
}

#[command]
#[description("roll the dice for giveaway")]
async fn dice_giveaway(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let gw = giveaway::get_giveway().await?;
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let winners_count = args.single::<u32>().unwrap_or(1);

  let mut winners = HashSet::new();

  let mut keys = gw.clone().into_keys().collect::<Vec<u64>>();
  let mut weights = gw.into_values().collect::<Vec<f64>>();
  let mut winner_counter = 0;
  let mut winners_strings: Vec<String> = vec![];

  loop {
    if !keys.is_empty() && !weights.is_empty() {
      // this is super dirty stupid hack, should be done other way
      let weights_clone = weights.clone();
      let winner_index = task::spawn_blocking(move || {
        if let Ok(dist) = WeightedIndex::new(&weights_clone) {
          let mut rng = rand::thread_rng();
          dist.sample(&mut rng)
        } else {
          0
        }
      }).await?;
      let winner = keys[winner_index];
      if !winners.contains(&winner) {
        let id = UserId( to_nzu!( winner ) );
        if let Ok(user) = ctx.http.get_user(id).await {
          winners.insert(winner);
          winners_strings.push(
            format!("{}: {winner}", user.name)
          );
          winner_counter += 1;
          if winner_counter == winners_count {
            break;
          }
        }
        keys.remove(winner_index);
        weights.remove(winner_index);
      }
    } else {
      break;
    }
  }

  let footer = format!("Requested by {}", msg.author.name);

  let eb = CreateEmbed::new()
    .color(0xe535ccu32)
    .title("Winners are:")
    .description(winners_strings.join("\n"))
    .thumbnail("https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png")
    .footer(CreateEmbedFooter::new(footer));

  msg.channel_id.send_message(ctx, CreateMessage::new()
    .embed(eb)
  ).await?;

  Ok(())
}

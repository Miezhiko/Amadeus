use crate::types::serenity::ReqwestClient;

use serenity::{
  prelude::*,
  builder::{ CreateMessage, CreateEmbed, CreateEmbedFooter },
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  }
};

use tokio::task;

use nipper::Document;

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

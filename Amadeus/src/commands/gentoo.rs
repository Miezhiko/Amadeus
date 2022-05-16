use crate::{
  types::serenity::ReqwestClient
};

use serenity::{
  prelude::*,
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
  let resp = reqwest_client.get(&url).send().await?.text().await?;

  let search_local = search.clone();
  let top_level = task::spawn_blocking(move || -> Vec<(String, String, String)> {
    let document = Document::from(&resp);
    document.nip("a > div").iter().take(5).flat_map(|element| {
      let text = element.text();
      let (atom, description) = text.split_once(' ')?;
      let (_category, pkgname) = atom.split_once('/')?;
      if pkgname.contains(&search_local) {
        Some((
          atom.to_string(),
          format!("http://gpo.zugaina.org/{atom}"),
          format!("**[{atom}](http://gpo.zugaina.org/{atom})** {description}")
        ))
      } else { None }
    }).collect::<Vec<(String, String, String)>>()
  }).await?;

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
  if let Err(why) = msg.channel_id.send_message(ctx, |m| {
    m.embed(|e|
      e.title(&search)
       .url(&url)
       .description(parse_result_str)
       .footer(|f| f.text(footer))
    );
    m
  }).await {
    msg.channel_id.say(ctx, &format!("Error: {why}")).await?;
  };

  Ok(())
}

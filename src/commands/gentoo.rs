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

use scraper::{ Html, Selector };

#[command]
#[description("Find package atom in Gentoo overlays")]
#[min_args(1)]
async fn zugaina(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let atom = args.message();
  let reqwest_client = {
    set!{ data            = ctx.data.read().await
        , reqwest_client  = data.get::<ReqwestClient>().unwrap() };
    reqwest_client.clone()
  };

  let url = format!("http://gpo.zugaina.org/Search?search={}", &atom);
  let resp = reqwest_client.get(&url).send().await?.text().await?;

  let parse_result = task::spawn_blocking(move || -> String {
    let fragment = Html::parse_fragment(&resp);
    let mut atoms_string: Vec<String> = vec![];
    if let Ok(selector) = Selector::parse("a > div") {
      for element in fragment.select(&selector).take(6) {
        let text = element.text().collect::<String>();
        let split2 = text.splitn(2, ' ').collect::<Vec<&str>>();
        if split2.len() > 1 {
          atoms_string.push(
            format!("**[{}](http://gpo.zugaina.org/{})**\n{}", split2[0]
                                                             , split2[0]
                                                             , split2[1])
          );
        }
      }
    }
    atoms_string.join("\n\n")
  }).await?;

  let footer = format!("Requested by {}", msg.author.name);
  if let Err(why) = msg.channel_id.send_message(ctx, |m| {
    m.embed(|e|
      e.title(&atom)
       .url(&url)
       .description(parse_result)
       .footer(|f| f.text(footer))
    );
    m
  }).await {
    msg.channel_id.say(ctx, why).await?;
  };

  Ok(())
}

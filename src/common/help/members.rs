use serenity::{
  prelude::*,
  model::channel::*,
  model::id::UserId,
  model::guild::Member,
};

use regex::Regex;
use once_cell::sync::Lazy;

pub async fn get_player(meme: &str, ctx: &Context, msg: &Message) -> eyre::Result<Member> {
  if meme.starts_with("<@") && meme.ends_with('>') {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("[<@!>]").unwrap() );
    let member_id = RE.replace_all(&meme, "").into_owned();
    let member = msg.guild_id.unwrap().member(
      ctx, UserId(member_id.parse::<u64>().unwrap())).await;
    match member {
      Ok(m) => Ok(m),
      Err(why) => Err(eyre!(why))
    }
  } else {
    let guild = &msg.guild(ctx).await.unwrap();
    let member_name = meme.split('#').next().unwrap();
    for m in guild.members.values() {
      if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
        m.user.name == member_name
      {
        return Ok(m.clone())
      }
    }
    Err(eyre!("can't find this player"))
  }
}

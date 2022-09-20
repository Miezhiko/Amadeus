use serenity::{
  prelude::*,
  model::channel::*,
  model::id::UserId,
  model::guild::Member,
};

use std::num::NonZeroU64;

use regex::Regex;
use once_cell::sync::Lazy;

use futures_util::{
  stream,
  StreamExt,
};

#[cfg(feature = "flo")]
pub async fn get_player(meme: &str, ctx: &Context, msg: &Message) -> anyhow::Result<Member> {
  if meme.starts_with("<@") && meme.ends_with('>') {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("[<@!>]").unwrap() );
    let member_id = RE.replace_all(meme, "").into_owned();
    let member = msg.guild_id.unwrap().member(
      ctx, UserId(member_id.parse::<NonZeroU64>().unwrap())).await;
    match member {
      Ok(m) => Ok(m),
      Err(why) => Err(anyhow!(why))
    }
  } else {
    if let Some(guild) = &msg.guild(&ctx.cache) {
      if let Some(member_name) = meme.split('#').next() {
        for m in guild.members.values() {
          if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
            m.user.name == member_name
          {
            return Ok(m.clone())
          }
        }
      }
    }
    Err(anyhow!("can't find this player"))
  }
}

pub async fn parse_member(ctx: &Context, msg: &Message, member_name: String) -> anyhow::Result<Member> {
  let mut members = Vec::new();
  if let Ok(id) = member_name.parse::<u64>() {
    let member = &msg.guild_id.unwrap().member(ctx, id).await;
    match member {
      Ok(m) => Ok(m.to_owned()),
      Err(why) => Err( anyhow!( why.to_string() )),
    }
  } else if member_name.starts_with("<@") && member_name.ends_with('>') {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("[<@!>]").unwrap());
    let member_id = RE.replace_all(&member_name, "").into_owned();
    let member = &msg.guild_id.unwrap().member(ctx, UserId(member_id.parse::<NonZeroU64>()?)).await;
    match member {
      Ok(m) => Ok(m.to_owned()),
      Err(why) => Err( anyhow!( why.to_string() )),
    }
  } else {
    let guild =
      if let Some(guild) = msg.guild(ctx.cache.as_ref()) {
        guild.clone()
      } else {
        return Err( anyhow!("can't get guild cache") );
      };
    let member_name = member_name.split('#').next().unwrap_or_default();
    for m in guild.members.values() {
      if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
        m.user.name == member_name
      {
        members.push(m);
      }
    }
    if members.is_empty() {
      let similar_members = &guild.members_containing(member_name, false, false);
      let mut members_string =  stream::iter(similar_members.iter())
        .map(|m| async move {
          let member = &m.0.user;
          format!("`{}`|", member.name)
        })
        .fold(String::new(), |mut acc, c| async move {
          acc.push_str(&c.await);
          acc
        }).await;
      let message = {
        if members_string.is_empty() {
          format!("No member named '{}' was found.", member_name.replace('@', ""))
        } else {
          members_string.pop();
          format!("No member named '{}' was found.\nDid you mean: {}", member_name.replace('@', ""), members_string.replace('@', ""))
        }
      };
      Err( anyhow!( message ))
    } else if members.len() == 1 {
      Ok(members[0].to_owned())
    } else {
      let mut members_string =  stream::iter(members.iter())
        .map(|m| async move {
          let member = &m.user;
          format!("`{}#{}`|", member.name, member.discriminator)
        })
        .fold(String::new(), |mut acc, c| async move {
          acc.push_str(&c.await);
          acc
        }).await;
      members_string.pop();
      let message = format!("Multiple members with the same name where found: '{}'", &members_string);
      Err( anyhow!(message) )
    }
  }
}

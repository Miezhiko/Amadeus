use serenity::{ model::channel::Message
              , builder::CreateMessage
              , prelude::* };

pub static MESSAGE_LIMIT: usize = 2000;

async fn serenity_direct_message_single(ctx: &Context, msg: &Message, text: &str) {
  if let Err(why) = msg.author.dm(ctx, CreateMessage::new().content(text)).await {
    error!("Error DMing user: {why}, text: {text}, user: {}", msg.author.name);
  }
}

async fn serenity_reply_single(ctx: &Context, msg: &Message, text: &str) {
  if text.starts_with(' ') {
    if let Err(why) = msg.reply(ctx, text.trim_start()).await {
      error!("Error replieng to user: {why}, text: {text}");
    }
  } else if let Err(why) = msg.reply(ctx, text).await {
    error!("Error replieng to user: {why}, text: {text}");
  }
}

async fn serenity_channel_message_single(ctx: &Context, msg: &Message, text: &str) {
  if let Err(why) = msg.channel_id.say(&ctx, text).await {
    error!("Error sending message to channel: {why}, text: {text}");
  }
}

async fn serenity_direct_message_multi(ctx: &Context, msg: &Message, texts: Vec<&str>) {
  for text in texts {
    serenity_direct_message_single(ctx, msg, text).await;
  }
}
async fn serenity_direct_message_multi2(ctx: &Context, msg: &Message, texts: Vec<String>) {
  for text in texts {
    serenity_direct_message_single(ctx, msg, &text).await;
  }
}

async fn serenity_reply_multi(ctx: &Context, msg: &Message, texts: Vec<&str>) {
  for text in texts {
    serenity_reply_single(ctx, msg, text).await;
  }
}
async fn serenity_reply_multi2(ctx: &Context, msg: &Message, texts: Vec<String>) {
  for text in texts {
    serenity_reply_single(ctx, msg, &text).await;
  }
}

async fn serenity_channel_message_multi(ctx: &Context, msg: &Message, texts: Vec<&str>) {
  for text in texts {
    serenity_channel_message_single(ctx, msg, text).await;
  }
}
async fn serenity_channel_message_multi2(ctx: &Context, msg: &Message, texts: Vec<String>) {
  for text in texts {
    serenity_channel_message_single(ctx, msg, &text).await;
  }
}

pub fn split_code(text: &str) -> Vec<String> {
  let default_split = text.as_bytes()
                          .chunks(MESSAGE_LIMIT - 8)
                          .map(|s| unsafe { ::std::str::from_utf8_unchecked(s) })
                          .collect::<Vec<&str>>();
  let mut to_top = false;
  let mut result = vec![];
  for part in default_split {
    let new_part = if !part.contains("```")
                   || part.matches("```").count() % 2 == 0 {
      if !to_top {
        part.to_owned()
      } else {
        to_top = part.contains("```") && part.matches("```").count() % 2 == 0;
        if to_top {
          format!("```\n{part}\n```")
        } else {
          format!("```\n{part}")
        }
      }
    } else if !to_top {
      to_top = true;
      format!("{part}\n```")
    } else {
      to_top = part.matches("```").count() % 2 == 0;
      if to_top {
        format!("```\n{part}\n```")
      } else {
        format!("```\n{part}")
      }
    };
    result.push(new_part);
  }
  result
}

pub fn split_message(text: &str) -> Vec<&str> {
  text.as_bytes()
    .chunks(MESSAGE_LIMIT)
    .map(|s| unsafe { ::std::str::from_utf8_unchecked(s) })
    .collect::<Vec<&str>>()
}

pub async fn direct_message(ctx: &Context, msg: &Message, text: &str) {
  if Message::overflow_length(text).is_some() {
    if text.contains("```") {
      serenity_direct_message_multi2(ctx, msg, split_code(text)).await;
    } else {
      serenity_direct_message_multi(ctx, msg, split_message(text)).await;
    }
  } else {
    serenity_direct_message_single(ctx, msg, text).await;
  }
}

pub async fn reply(ctx: &Context, msg: &Message, text: &str) {
  if Message::overflow_length(text).is_some() {
    if text.contains("```") {
      serenity_reply_multi2(ctx, msg, split_code(text)).await;
    } else {
      serenity_reply_multi(ctx, msg, split_message(text)).await;
    }
  } else {
    serenity_reply_single(ctx, msg, text).await;
  }
}

pub async fn channel_message(ctx: &Context, msg: &Message, text: &str) {
  if Message::overflow_length(text).is_some() {
    if text.contains("```") {
      serenity_channel_message_multi2(ctx, msg, split_code(text)).await;
    } else {
      serenity_channel_message_multi(ctx, msg, split_message(text)).await;
    }
  } else {
    serenity_channel_message_single(ctx, msg, text).await;
  }
}

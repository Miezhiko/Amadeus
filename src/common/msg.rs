use serenity::{ model::channel::Message
              , prelude::* };

pub static MESSAGE_LIMIT: usize = 2000;

async fn serenity_direct_message_single(ctx: &Context, msg: &Message, text: &str) {
  if let Err(why) = msg.author.dm(ctx, |m| m.content(text)).await {
    error!("Error DMing user: {why}");
  }
}

async fn serenity_reply_single(ctx: &Context, msg: &Message, text: &str) {
  if text.starts_with(' ') {
    if let Err(why) = msg.reply(ctx, text.trim_start()).await {
      error!("Error replieng to user: {why}");
    }
  } else if let Err(why) = msg.reply(ctx, text).await {
    error!("Error replieng to user: {why}");
  }
}

async fn serenity_channel_message_single(ctx: &Context, msg: &Message, text: &str) {
  if let Err(why) = msg.channel_id.say(&ctx, text).await {
    error!("Error sending message to channel: {why}");
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
  if let Some(first_space) = text.find(' ') {
    let start_from =
      if let Some(first_newline) = text.find('\n') {
        if first_space < first_newline { first_space }
        else { first_newline }
      } else { first_space };
    let starting_pattern = &text[..start_from];
    let whole_new_text = &text[start_from..text.len()-4];
    let peaces = whole_new_text.as_bytes()
      .chunks(MESSAGE_LIMIT - 200)
      .map(|s| unsafe { ::std::str::from_utf8_unchecked(s).replace("```", "'''") });
    peaces.map(|s| format!("{starting_pattern}\n{s}\n```")).collect()
  } else {
    vec![text.to_string()]
  }
}

pub fn split_message(text: &str) -> Vec<&str> {
  text.as_bytes()
    .chunks(MESSAGE_LIMIT)
    .map(|s| unsafe { ::std::str::from_utf8_unchecked(s) })
    .collect::<Vec<&str>>()
}

pub async fn direct_message(ctx: &Context, msg: &Message, text: &str) {
  if Message::overflow_length(text).is_some() {
    if text.starts_with("```") {
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
    if text.starts_with("```") {
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
    if text.starts_with("```") {
      serenity_channel_message_multi2(ctx, msg, split_code(text)).await;
    } else {
      serenity_channel_message_multi(ctx, msg, split_message(text)).await;
    }
  } else {
    serenity_channel_message_single(ctx, msg, text).await;
  }
}

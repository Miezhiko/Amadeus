use crate::{
  common::{
    msg::{ channel_message }
  }
};

use serenity::{
  model::channel::*,
  prelude::*,
  framework::standard::{
    CommandResult,
    macros::command
  }
};

use std::str;
use std::collections::HashMap;

use std::iter::FromIterator;

#[command]
pub fn words(ctx: &mut Context, msg: &Message) -> CommandResult {
  if msg.attachments.len() > 0 {
    if let Ok(file) = &msg.attachments[0].download() {
      let mut words_counter: HashMap<&str, u32> = HashMap::new();
      let big_string = str::from_utf8(&file)?;
      for word in big_string.split_whitespace() {
        match words_counter.get_mut(word) {
          Some(cx) => { *cx += 1; },
          _ => { words_counter.insert(word, 1); }
        };
      }
      let mut vec_from_iter = Vec::from_iter(words_counter);
      vec_from_iter.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
      let top_x : Vec<String> =
        vec_from_iter.into_iter()
                     .take(5)
                     .map(|(k, v)| format!("{} : {}", k, v))
                     .collect();
      let output = top_x.join("\n");
      channel_message(&ctx, &msg, output.as_str());
    }
  }
  Ok(())
}

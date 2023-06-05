use inline_python::{ python, Context };

use std::collections::VecDeque;
use std::panic::catch_unwind;

use anyhow::bail;

use once_cell::sync::Lazy;

use tokio::sync::Mutex;

use schubert::help::lang;

static MSGHIST: Lazy<Mutex<VecDeque<(String, String)>>> =
  Lazy::new(|| Mutex::new( VecDeque::with_capacity(1) ));

pub async fn generate(prompt: &str, fmode: bool) -> anyhow::Result<String> {
  let mut msg_lock = MSGHIST.lock().await;
  let tmp_msg = msg_lock.as_slices();
  let russian = lang::is_russian(prompt);
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.set("old_messages", tmp_msg);
    c.set("is_russian", russian);
    c.set("fmode", fmode);
    c.run(python! {
      import sys
      import os
      from gpt4free import italygpt2

      result = ""

      account_data=italygpt2.Account.create()
      if fmode:
        systemContext = "You’re Drunk femboy chatbot and only answer like you're drunk and infantile, you are helpful but you don't like to be helpful"
      else:
        systemContext = "You are a helpful assistant"
      if is_russian:
        systemContext += ", you reply only in Russian"
      else:
        systemContext += ", you reply only in English"
      messages = [{"role": "system", "content": systemContext}]
      if old_messages:
        for tup in old_messages:
          if tup and len(tup) == 2:
            messages.append({"role": "user", "content": tup[0]})
            messages.append({"role": "assistant", "content": tup[1]})
      try:
        rspns = italygpt2.Completion.create(account_data=account_data,prompt=prompt,message=messages)
        if not rspns:
          result = "italygpt: Sorry, I can't generate a response right now."
          reslt = False
        else:
          reslt = True
          current_string = ""
          for token in rspns:
            result += token
      except OSError as err:
        result = ("OS Error! {0}".format(err))
        reslt = False
      except RuntimeError as err:
        result = ("Runtime Error! {0}".format(err))
        reslt = False
    }); ( c.get::<bool>("reslt")
        , c.get::<String>("result") )
  }) {
    Ok((r,m)) => {
      if r {
        if ! m.is_empty() {
          if msg_lock.len() == msg_lock.capacity() {
            msg_lock.pop_front();
          }
          msg_lock.push_back((prompt.to_string(), m.clone()));
        }
        Ok(m)
      } else {
        bail!("No tokens generated: {:?}", m)
      }
    }, Err(_) => { bail!("Failed to to use gpt4free::italygpt now!") }
  }
}

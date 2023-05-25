use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

use once_cell::sync::Lazy;

use tokio::sync::Mutex;

use mozart::help::lang;

static MYMSG: Lazy<Mutex<String>> =
  Lazy::new(|| Mutex::new( String::from("") ));

static TOKEN: Lazy<Mutex<String>> =
  Lazy::new(|| Mutex::new( String::from("") ));

pub async fn generate(prompt: &str, fmode: bool) -> anyhow::Result<String> {
  let mut msg_lock  = MYMSG.lock().await;
  let tmp_msg       = msg_lock.as_str();
  let mut token     = TOKEN.lock().await;
  let token_tmp     = token.as_str();
  let russian       = lang::is_russian(prompt);
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.set("message_id", tmp_msg);
    c.set("mytoken", token_tmp);
    c.set("fmode", fmode);
    c.set("is_russian", russian);
    c.run(python! {
      import sys
      import os
      from gpt4free import usesless

      if fmode:
        systemContext = "You’re Drunk femboy chatbot and only answer like you're drunk and infantile, you are helpful but you don't like to be helpful"
      else:
        systemContext = "You are a helpful assistant"
      if is_russian:
        systemContext += ", you reply only in Russian"
      else:
        systemContext += ", you reply in English"

      result = ""
      try:
        if not mytoken:
          mytoken = usesless.Account.create(logging=True)
        rspns = usesless.Completion.create(prompt=prompt, systemMessage=systemContext, parentMessageId=message_id, token=mytoken)
        if not rspns:
          result = "useless: Sorry, I can't generate a response right now."
          reslt = False
        else:
          reslt = True
          result = rspns["text"]
          message_id = rspns["id"]
      except OSError as err:
        result = ("OS Error! {0}".format(err))
        reslt = False
      except RuntimeError as err:
        result = ("Runtime Error! {0}".format(err))
        reslt = False
    }); ( c.get::<bool>("reslt")
        , c.get::<String>("result")
        , c.get::<String>("message_id")
        , c.get::<String>("mytoken") )
  }) {
    Ok((r, m, new_msg_id, new_token)) => {
      if r {
        *msg_lock = new_msg_id;
        *token    = new_token;
        Ok(m)
      } else {
        bail!("No tokens generated: {:?}", m)
      }
    }, Err(_) => { bail!("Failed to to use gpt4free::useless now!") }
  }
}

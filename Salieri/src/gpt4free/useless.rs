use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

use once_cell::sync::Lazy;

use tokio::sync::Mutex;

static MYMSG: Lazy<Mutex<String>> =
  Lazy::new(|| Mutex::new( String::from("") ));

static TOKEN: Lazy<Mutex<String>> =
  Lazy::new(|| Mutex::new( String::from("") ));

pub async fn generate(prompt: &str) -> anyhow::Result<Vec<String>> {
  let mut msg_lock  = MYMSG.lock().await;
  let tmp_msg       = msg_lock.as_str();
  let mut token     = TOKEN.lock().await;
  let token_tmp     = token.as_str();
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.set("message_id", tmp_msg);
    c.set("mytoken", token_tmp);
    c.run(python! {
      import sys
      import os
      from gpt4free import usesless

      result = []
      try:
        if not mytoken:
          mytoken = usesless.Account.create(logging=True)
        rspns = usesless.Completion.create(prompt=prompt, parentMessageId=message_id, token=mytoken)
        if not rspns:
          result = ["useless: Sorry, I can't generate a response right now."]
          reslt = False
        else:
          reslt = True
          current_string = ""
          for token in rspns["text"]:
            current_string += token
            if len(current_string) >= 1980:
              result.append(current_string[:1980])
              current_string = current_string[1980:]
          if current_string:
            result.append(current_string)
          message_id = rspns["id"]
      except OSError as err:
        result = [("OS Error! {0}".format(err))]
        reslt = False
      except RuntimeError as err:
        result = [("Runtime Error! {0}".format(err))]
        reslt = False
    }); ( c.get::<bool>("reslt")
        , c.get::<Vec<String>>("result")
        , c.get::<String>("message_id")
        , c.get::<String>("mytoken") )
  }) {
    Ok((r, m, new_msg, new_token)) => {
      if r {
        *msg_lock = new_msg;
        *token    = new_token;
        Ok(m)
      } else {
        bail!("No tokens generated: {:?}", m)
      }
    }, Err(_) => { bail!("Failed to to use gpt4free::useless now!") }
  }
}

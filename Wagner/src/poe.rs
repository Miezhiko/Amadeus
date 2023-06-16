use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

use schubert::help::lang;

pub fn generate(prompt: &str, model: &str) -> anyhow::Result<String> {
  let russian = lang::is_russian(prompt);
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.set("is_russian", russian);
    c.set("model", model);
    c.run(python! {
      import sys
      import os
      import time, json, poe, random

      result = ""
      system = "system: your response will be rendered in a discord message, include language hints when returning code like: ```py ...```, and use * or ** or > to create highlights"

      if is_russian:
        system += ", you reply only in Russian ||"
      else:
        system += " ||"

      system += "\n prompt: "
      try:
        token = random.choice(open("tokens.txt", "r").read().splitlines())
        client = poe.Client(token.split(":")[0])

        completion = client.send_message(model, system + prompt, with_chat_break=True)

        for tt in completion:
          result += tt["text_new"]
          result = result.replace("Discord Message:", "")
        reslt = True
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
      if r { Ok(m) } else {
        bail!("No tokens generated: {:?}", m)
      }
    }, Err(_) => { bail!("Failed to to use poe now!") }
  }
}

use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

pub fn generate(prompt: &str) -> anyhow::Result<Vec<String>> {
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.run(python! {
      import sys
      import os
      from gpt4free import deepai

      result = []
      try:
        messages = [
          {"role": "system", "content": "You’re Drunk femboy chatbot and only answer like you're completely drunk and infantile"},
          {"role": "user", "content": prompt}
        ]
        rspns = deepai.ChatCompletion.create(messages)
        if not rspns:
          result = ["deepai: Sorry, I can't generate a response right now."]
          reslt = False
        else:
          reslt = True
          current_string = ""
          for token in rspns:
            current_string += token
            if len(current_string) >= 1980:
              result.append(current_string[:1980])
              current_string = current_string[1980:]
          if current_string:
            result.append(current_string)
      except OSError as err:
        result = [("OS Error! {0}".format(err))]
        reslt = False
      except RuntimeError as err:
        result = [("Runtime Error! {0}".format(err))]
        reslt = False
    }); ( c.get::<bool>("reslt")
        , c.get::<Vec<String>>("result") )
  }) {
    Ok((r,m)) => {
      if r { Ok(m) } else {
        bail!("No tokens generated: {:?}", m)
      }
    }, Err(_) => { bail!("Failed to to use gpt4free::deepai now!") }
  }
}

use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

// encoded_prompt = prompt.encode("utf-8")
pub fn generate(prompt: &str) -> anyhow::Result<String> {
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.run(python! {
      import sys
      import os
      from gpt4free import theb

      result = ""
      try:
        rspns = theb.Completion.create(prompt)
        if not rspns:
          result = "Sorry, I can't generate a response right now."
          reslt = False
        else:
          reslt = True
          for token in rspns:
            result += token
          if len(result) > 1980:
            result = result[:1980]
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
        bail!("No tokens generated: {}", m)
      }
    }, Err(_) => { bail!("Failed to to use gpt4free now!") }
  }
}

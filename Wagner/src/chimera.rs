use inline_python::{ python, Context };

use std::panic::catch_unwind;

use anyhow::bail;

pub fn generate(prompt: &str) -> anyhow::Result<String> {
  match catch_unwind(|| {
    let c = Context::new();
    c.set("prompt", prompt);
    c.run(python! {
      import sys
      import os
      import openai

      result = ""
      try:
        with open("chimera.txt", "r") as file:
          token = file.readline().strip()

        openai.api_key = token
        openai.api_base = "https://chimeragpt.adventblocks.cc/v1"

        response = openai.ChatCompletion.create(
          model="gpt-4-0613",
          messages=[
            {"role": "user", "content": prompt},
          ]
        )
        rspns = response["choices"]

        if not rspns:
          result = "chimera: Sorry, I can't generate a response right now."
          reslt = False
        else:
          reslt = True
          reslt = rspns[0]["message"]["content"]
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
    }, Err(_) => { bail!("Failed to to use chimera now!") }
  }
}

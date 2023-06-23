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
      import g4f

      result = ""
      try:
        provider = g4f.Provider.Yqcloud
        rspns = g4f.ChatCompletion.create(model="gpt-4", 
                          messages=[{"role": "user", 
                                     "content": prompt}], 
                          stream=False, 
                          provider=provider)
        if not rspns:
          result = "yqcloud: Sorry, I can't generate a response right now."
          reslt = False
        else:
          reslt = True
          reslt = rspns
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
    }, Err(_) => { bail!("Failed to to use g4f::yqcloud now!") }
  }
}

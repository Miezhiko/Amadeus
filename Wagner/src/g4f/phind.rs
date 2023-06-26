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
        import json
        import time
        import subprocess
        import sys
        import json
        import datetime
        import urllib.parse
        
        from curl_cffi import requests
        from g4f.typing import sha256, Dict, get_type_hints
        
        url = "https://phind.com"
        model = ["gpt-4"]
        supports_stream = True
        
        def create_completion(model: str, messages: list, stream: bool):
            config = json.dumps({
                "model": model,
                "messages": messages}, separators=(",", ":"))
            cmd = ["python3", "/data/contrib/rust/Amadeus/misc/phind.py", config]
            p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            for line in iter(p.stdout.readline, b""):
                if b"<title>Just a moment...</title>" in line:
                    os.system("clear" if os.name == "posix" else "cls")
                    yield "Clouflare error, please try again..."
                    os._exit(0)
                else:
                    if b"ping - 2023-" in line:
                        continue
                    yield line.decode("utf-8")

        rspns = "".join(create_completion(model="gpt-4",
                          messages=[{"role": "user", 
                                     "content": prompt}], 
                          stream=False))
        if not rspns:
          result = "phind: Sorry, I can't generate a response right now."
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
    }, Err(_) => { bail!("Failed to to use g4f::phind now!") }
  }
}

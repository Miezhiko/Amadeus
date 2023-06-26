import g4f
import sys


import os
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

    path = "/data/contrib/rust/Amadeus/misc"
    config = json.dumps({
        "model": model,
        "messages": messages}, separators=(",", ":"))

    cmd = ["python3", f"{path}/phind.py", config]

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

response = "".join(create_completion(model="gpt-4", #g4f.ChatCompletion.create(model="gpt-4", 
                  messages=[{"role": "user", 
                      "content": "Who are you? What version of GPT you use?"}], 
                  stream=False))

print(response)

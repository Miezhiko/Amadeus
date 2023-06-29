import sys
import os
import time, json, poe, random

is_russian = False
prompt = "who are you?"

result = ""
system = "system: your response will be rendered in a discord message, include language hints when returning code like: ```py ...```, and use * or ** or > to create highlights"

if is_russian:
    system += ", you reply only in Russian ||"
else:
    system += " ||"

system += "\n prompt: "
try:
    token = random.choice(open("tokens.txt", "r").read().splitlines())
    client = poe.Client(token)

    print(json.dumps(client.bot_names, indent=2))

    completion = client.send_message("capybara", system + prompt, with_chat_break=True)

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

print(result)

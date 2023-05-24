#!/usr/bin/python3

import sys
import os

from gpt4free import usesless

message_id = ""
while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    mytoken = usesless.Account.create(logging=True)
    req = rspns = usesless.Completion.create(prompt=prompt, parentMessageId=message_id, token=mytoken)

    print(f"Answer: {{req[\"text\"]}}")
    message_id = req["id"]

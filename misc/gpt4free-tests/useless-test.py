#!/usr/bin/python3

import sys
import os

from gpt4free import usesless

message_id = ""
while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    req = usesless.Completion.create(prompt=prompt, parentMessageId=message_id)

    print(f"Answer: {{req[\"text\"]}}")
    message_id = req["id"]

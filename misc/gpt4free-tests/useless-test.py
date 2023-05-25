#!/usr/bin/python3

import sys
import os

from gpt4free import usesless

mytoken = usesless.Account.create(logging=True)

message_id = ""
while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    systemContext = "You are a helpful assistant"

    req = usesless.Completion.create(prompt=prompt, systemMessage=systemContext, parentMessageId=message_id, token=mytoken)

    ans = req["text"]
    print(f"Answer: {ans}")
    message_id = req["id"]

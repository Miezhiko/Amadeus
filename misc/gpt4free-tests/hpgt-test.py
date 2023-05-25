#!/usr/bin/python3

import sys
import os

from gpt4free import gptworldAi

message_id = ""
while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    for chunk in gptworldAi.Completion.create(prompt, ""):
        print(chunk, end="", flush=True)
        print()

    #print(f"Answer: {req}")


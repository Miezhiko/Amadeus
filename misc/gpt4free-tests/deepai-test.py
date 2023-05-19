#!/usr/bin/python3

import sys
import os

from gpt4free import deepai

while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    response = deepai.Completion.create(prompt)

    if not response:
        response = "Sorry, I can't generate a response right now."
    token_count = 0
    result = ""
    for token in response:
        token_count += 1
        result += token

    print(f"Answer: {result}")

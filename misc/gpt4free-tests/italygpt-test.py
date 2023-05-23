#!/usr/bin/python3

import sys
import os

from gpt4free import italygpt2

account_data=italygpt2.Account.create()

message_id = ""
while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    messages = [
        {"role": "system", "content": "You’re a cat and you answer like a cat"},
    ]

    #response = italygpt2.Completion.create(account_data=account_data, prompt=prompt)
    response = italygpt2.Completion.create(account_data=account_data,prompt=prompt,message=messages)

    if not response:
        response = "Sorry, I can't generate a response right now."
    token_count = 0
    result = ""
    for token in response:
        token_count += 1
        result += token

    print(result)

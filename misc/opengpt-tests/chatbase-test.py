#!/usr/bin/python3

import sys
import os

from opengpt.models.completion.chatbase.model import Model

chatbase = Model()

while True:
    prompt = input("Question: ")
    if prompt == "!stop":
        break

    print(chatbase.GetAnswer(prompt=prompt, model="gpt-4"))

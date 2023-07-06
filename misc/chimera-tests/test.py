import openai

with open('chimera.txt', 'r') as file:
  token = file.readline().strip()

openai.api_key = token
openai.api_base = "https://chimeragpt.adventblocks.cc/v1"

response = openai.ChatCompletion.create(
  model='gpt-4',
  messages=[
    {'role': 'user', 'content': "what is your gpt version?"},
  ]
)

choices = response["choices"]
if choices:
  print(choices[0]["message"]["content"])

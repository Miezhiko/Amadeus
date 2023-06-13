import time, json, poe, random

models = {
    'claude-v1': 'a2_2',
    'claude-instant': 'a2',
    'claude-instant-100k': 'a2_100k',
    'sage': 'capybara',
    'gpt-4': 'beaver',
    'gpt-3.5-turbo': 'chinchilla',
}

prompt = 'who are you?'

model = 'gpt-4'
base = f'\n'
system = 'system: your response will be rendered in a discord message, include language hints when returning code like: ```py ...```, and use * or ** or > to create highlights ||\n prompt: '

token = random.choice(open('/data/contrib/rust/Amadeus/tokens.txt', 'r').read().splitlines())
client = poe.Client(token)

completion = client.send_message(models[model], system + prompt, with_chat_break=True)

for tt in completion:
    base += tt['text_new']
    base = base.replace('Discord Message:', '')

print(base)

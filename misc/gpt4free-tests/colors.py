from gpt4free import aicolors

prompt = "show the colour of sad communist comrade who is working all day long but losing money on intel stock"
req = aicolors.Completion.create(prompt)

print(req)

import g4f

# normal response
response = g4f.ChatCompletion.create( model=..., messages=[
                                     {"role": "system", "content": "you answer what version of GPT you use"},
                                     {"role": "user", "content": "what version of GPT you use?"}]
                                     , stream=False
                                     , provider=g4f.Provider.... )

print(response)

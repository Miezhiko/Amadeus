import g4f

# normal response
response = g4f.ChatCompletion.create( model=g4f.Model.gpt_4, messages=[
                                     {"role": "system", "content": "you answer what version of GPT you use"},
                                     {"role": "user", "content": "what version of GPT you use?"}]
                                     , stream=False
                                     , provider=g4f.Provider.EasyChat )

print(response)

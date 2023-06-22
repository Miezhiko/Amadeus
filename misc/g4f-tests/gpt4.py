import g4f
import sys

# Provider selection
provider=g4f.Provider.Phind

# Streaming is not supported by these providers
if provider in {g4f.Provider.Aws, g4f.Provider.Ora, 
				g4f.Provider.Bard, g4f.Provider.Aichat}:
	stream=False
else:
	stream=True

print(provider.params) # supported args

# Getting the response
response = g4f.ChatCompletion.create(model='gpt-4', 
									messages=[{"role": "user", 
											"content": "Who are you?"}], 
									stream=stream, 
									provider=provider)

# Printing the response          
if stream:          
	for message in response:
		print(message, end="")
		sys.stdout.flush()
	print("\n")
else:
	print(response)

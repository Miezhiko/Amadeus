##REALLY PLANNED:

 - CORRECTLY SPLIT MESSAGES WITH CODE (IT WAS DONE SOMEWHERE BUT IS NOT WORKING ON NEEDED TASKS) 
 - Drop Salieri and Strauss because Celery design for compiled language is horribly bad (you need to link your processing code with both service and client)
 - Switch to Kafka or alternatively write own AMQ based proto
 - Change Kafka keys, use numbers instead of strings
 - Flag for minimal upgrade without deps updates for safety
 - Move some stuff to separated repository (maybe?)
 - Ability to specify desired gpt service from Kalmarity
 - Single interface for bert models processing on strauss
 - Store long-term conversation with everyone in DB, use Summarize model for context generation
   (Run Summarization generation periodically, possibly on deep nights)
   (Use generated context for Bert models later)

##I DON'T WANT TO DO IT (contributions welcome):

 - Stream announce (e.g. stream in 30 mins or something like that) + stream announce role
 - Select menu roles
 - Some of slash commands are still missing (vs command, etc...)
 - I need part of speech tagging POSModel but for russian langauge, that could be used for Kathoey
 - Finetune GTP2 model
 - Documentation
 - Rework config files for simpler/wider usage
 - Youtube streams support

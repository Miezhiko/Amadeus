##REALLY PLANNED:

 - Restructure team_checker module (code quality)
 - Some of slash commands are still missing (vs command, etc...)
 - Stream announce (e.g. steam in 30 mins or something like that) + stream announce role
 - Use Summarization model for large texts and just sometimes with context cache!
 - Logging for memory changes.

```rust
    let summarization_model = SummarizationModel::new(Default::default())?;
    let input = ["I"];
    let output = summarization_model.summarize(&input);
```

 - SL World API: http://wiki.secondlife.com/wiki/World_API

##I DON'T WANT TO DO IT:

 - I need part of speech tagging POSModel but for russian langauge, that could be used for Kathoey
 - Finetune GTP2 model
 - Documentation
 - Rework config files for simpler/wider usage

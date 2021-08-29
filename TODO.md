##REALLY PLANNED:

 - emoji roles
 - restructure team_checker module (code quality)
 - some of slash commands are still missing (vs command, etc...)
 - stream announce (e.g. steam in 30 mins or something like that) + stream announce role
 - display player pings for 2x2 when live.
 - use Summarization model for large texts and just sometimes with context cache!

```rust
    let summarization_model = SummarizationModel::new(Default::default())?;
    let input = ["I"]
    let output = summarization_model.summarize(&input);
```

##I DON'T WANT TO DO IT:

 - I need part of speech tagging POSModel but for russian langauge, that could be used for Kathoey
 - finetune GTP2 model
 - documentation

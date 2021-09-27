##REALLY PLANNED:

 - restructure team_checker module (code quality)
 - some of slash commands are still missing (vs command, etc...)
 - stream announce (e.g. steam in 30 mins or something like that) + stream announce role
 - use Summarization model for large texts and just sometimes with context cache!

```rust
    let summarization_model = SummarizationModel::new(Default::default())?;
    let input = ["I"];
    let output = summarization_model.summarize(&input);
```

 - SL World API: http://wiki.secondlife.com/wiki/World_API

##I DON'T WANT TO DO IT:

 - I need part of speech tagging POSModel but for russian langauge, that could be used for Kathoey
 - finetune GTP2 model
 - documentation
 - rework config files for simpler/wider usage

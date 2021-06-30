use crate::steins::ai::bert::TRANSLATION_LIMIT;

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use rust_bert::pipelines::translation::{ Language
                                       , TranslationConfig
                                       , TranslationModel };

use tch::Device;
use tokio::task;

async fn bert_translate(_ctx: &Context, text: String, lang: Language)
          -> anyhow::Result<String> {
  task::spawn_blocking(move || {
    let mut something = text;
    if something.len() > TRANSLATION_LIMIT {
      if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
        something = something[i..].to_string();
      }
    }

    let translation_config =
      TranslationConfig::new(lang, Device::cuda_if_available());

    let model = TranslationModel::new(translation_config)?;

    let output = model.translate(&[something.as_str()]);
    if output.is_empty() {
      Err(anyhow!("Failed to translate with TranslationConfig EnglishToRussian"))
    } else {
      let translation = &output[0];
      Ok(translation.clone())
    }
  }).await?
}

#[command]
#[min_args(1)]
#[description("Translate English to Russian")]
#[aliases(перевод, en2ru)]
pub async fn perevod(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **English** to **Russian**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::EnglishToRussian).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
             .author(|a| a.icon_url(&msg.author.face())
                          .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate Russian to English")]
#[aliases(ru2en)]
pub async fn translate(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **Russian** to **English**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::RussianToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
             .author(|a| a.icon_url(&msg.author.face())
                          .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate English to German")]
async fn en2de(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **English** to **German**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::EnglishToGerman).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
            .author(|a| a.icon_url(&msg.author.face())
                         .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate German to English")]
async fn de2en(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **German** to **English**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::GermanToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
            .author(|a| a.icon_url(&msg.author.face())
                         .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate English to French")]
async fn en2fr(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **English** to **French**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::EnglishToFrench).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
            .author(|a| a.icon_url(&msg.author.face())
                         .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate French to English")]
async fn fr2en(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  let fields = vec![
    ("Text", format!("{}\n", text), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **French** to **English**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(ctx, text.clone(), Language::FrenchToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      if let Ok(mut mm) = mmm {
        mm.edit(ctx, |m|
          m.embed(|e|
            e.fields(fields)
            .author(|a| a.icon_url(&msg.author.face())
                         .name(&msg.author.name)
                    )
          )
        ).await?;
      }
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

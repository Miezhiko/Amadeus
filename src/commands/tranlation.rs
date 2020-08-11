use serenity::{
  prelude::*,
  model::channel::*,
  model::gateway::Activity,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use rust_bert::pipelines::translation::{ Language, TranslationConfig, TranslationModel };

use tch::Device;
use tokio::task;

async fn bert_translate(ctx: &Context, text: String, lang: Language)
          -> failure::Fallible<String> {
  ctx.set_activity(Activity::listening("Translating!")).await;
  ctx.idle().await;
  let result = task::spawn_blocking(move || {
      let translation_config =
        TranslationConfig::new(lang, Device::cuda_if_available());

      let model = TranslationModel::new(translation_config)?;

      let output = model.translate(&[text.as_str()]);
      if output.is_empty() {
        error!("Failed to translate with TranslationConfig EnglishToRussian");
        Ok(text)
      } else {
        let translation = &output[0];
        Ok(translation.clone())
      }
    }).await.unwrap();
  ctx.online().await;
  result
}

#[command]
#[min_args(1)]
async fn en2ru(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::EnglishToRussian).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **English** to **Russian**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn ru2en(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::RussianToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **Russian** to **English**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn en2de(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::EnglishToGerman).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **English** to **German**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn de2en(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::GermanToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **German** to **English**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn en2fr(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::EnglishToFrench).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **English** to **French**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
async fn fr2en(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command {:?}", why);
  }
  match bert_translate(ctx, text.clone(), Language::FrenchToEnglish).await {
    Ok(out) => {
      let fields = vec![
        ("Original Text", format!("{}\n", text), false),
        ("Translation", out, false)
      ];
      msg.channel_id.send_message(ctx, |m| {
        m.content("From **French** to **English**");
        m.embed(|e| {
          e.fields(fields)
           .author(|a| a.icon_url(&msg.author.face())
                        .name(msg.author.name.as_str())
                  )
        })
      }).await?;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

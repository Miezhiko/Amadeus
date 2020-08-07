use crate::{
  common::msg::channel_message
};

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

use failure;

use tokio::task;

async fn bert_translate(ctx: &Context, text: String, lang: Language)
          -> failure::Fallible<String> {
  ctx.set_activity(Activity::listening("Translating")).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::EnglishToRussian).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::RussianToEnglish).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::EnglishToGerman).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::GermanToEnglish).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::EnglishToFrench).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
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
  let text = args.raw().collect::<Vec<&str>>().join(" ");
  match bert_translate(ctx, text, Language::FrenchToEnglish).await {
    Ok(out) => {
      channel_message(ctx, msg, out.as_str()).await;
    },
    Err(why) => {
      error!("Translation failed: {:?}", why);
    }
  }
  Ok(())
}

use serenity::{
  prelude::*,
  model::channel::*,
  framework::standard::{
    CommandResult, Args,
    macros::command
  },
};

use mozart::bert::translation::{ bert_translate, SLanguage };

#[command]
#[min_args(1)]
#[description("Translate English to Russian")]
#[aliases(перевод, en2ru)]
pub async fn perevod(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::English, SLanguage::Russian).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate Ukrainian to Russian")]
pub async fn ua2ru(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **Ukrainian** to **Russian**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(text.clone(), SLanguage::Ukrainian, SLanguage::Russian).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
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
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::Russian, SLanguage::English).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
    }
  }
  Ok(())
}

#[command]
#[min_args(1)]
#[description("Translate Russian to Ukrainian")]
pub async fn ru2ua(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
  let text = args.message().to_string();
  if let Err(why) = msg.delete(&ctx).await {
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
  ];
  let mmm = msg.channel_id.send_message(ctx, |m|
            m.embed(|e|
             e.title("Translating From **Russian** to **Ukrainian**...")
              .fields(fields)
              .author(|a| a.icon_url(&msg.author.face())
                           .name(&msg.author.name)
                      )
            )
          ).await;
  match bert_translate(text.clone(), SLanguage::Russian, SLanguage::Ukrainian).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
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
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::English, SLanguage::German).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
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
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::German, SLanguage::English).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
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
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::English, SLanguage::French).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
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
    error!("Error deleting original command, {why}");
  }
  let fields = vec![
    ("Text", format!("{text}\n"), false),
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
  match bert_translate(text.clone(), SLanguage::French, SLanguage::English).await {
    Ok(out) => {
      let fields = vec![
        ("Text", format!("{text}\n"), false),
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
      error!("Translation failed: {why}");
    }
  }
  Ok(())
}

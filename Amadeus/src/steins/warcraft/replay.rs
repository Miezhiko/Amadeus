use crate::{
  types::serenity::ReqwestClient,
  common::{ msg::channel_message
          , constants::APM_PICS
          , colors::gen_colors
          },
  collections::team::PLAYERS,
  steins::warcraft::{
    aka_checker::check_aka,
    w3g::analyze
  }
};

use serenity::{
  prelude::*,
  builder::*,
  model::channel::{ Attachment
                  , Message
                  , ReactionType },
  builder::CreateEmbed
};

use std::{
  time::Duration,
  borrow::Borrow
};

use async_std::{ fs::File, fs
               , prelude::* };

use plotters::prelude::*;

pub async fn replay_embed( ctx: &Context
                         , msg: &Message
                         , file: &Attachment ) -> anyhow::Result<()> {
  let fname_apm = format!("{}_apm.png", &file.filename);
  info!("Downloading replay");
  if let Ok(bytes) = file.download().await {
    let mut fw3g = match File::create(&file.filename).await {
      Ok(replay) => replay,
      Err(why) => {
        channel_message(ctx, msg, "Error getting replay").await;
        return Err(anyhow!("Error creating file: {why}"));
      }
    };
    if let Err(why) = fw3g.write_all(&bytes).await {
      if let Err(why2) = fs::remove_file(&file.filename).await {
        error!("Error removing file: {why2}");
      }
      return Err(anyhow!("Error writing to file: {why}"));
    }
    let _ = fw3g.sync_data().await;
    info!("Parsing replay");
    let data_maybe = analyze(&file.filename, false).await;
    if let Err(why) = data_maybe {
      if let Err(why2) = fs::remove_file(&file.filename).await {
        error!("Error removing file: {why2}");
      }
      return Err(anyhow!("Corrupted replay file? {why}"));
    }
    let (d, flds) = data_maybe?;
    setm!{ eb1 = CreateEmbed::new()
         , eb2 = CreateEmbed::new()
         , eb3 = CreateEmbed::new() };
    let footer = format!("Uploaded by {}", msg.author.name);
    eb1 = eb1.color(0xe535ccu32);     eb2 = eb2.color(0xe535ccu32);     eb3 = eb3.color(0xe535ccu32);
    eb1 = eb1.title(&file.filename);  eb2 = eb2.title(&file.filename);  eb3 = eb3.title(&file.filename);
    eb1 = eb1.description(&d);        eb2 = eb2.description("units");   eb3 = eb3.description("APM Graph");
    static AMADEUS_LOGO: &str = "https://vignette.wikia.nocookie.net/steins-gate/images/0/07/Amadeuslogo.png";
    eb1 = eb1.thumbnail(AMADEUS_LOGO);
    eb2 = eb2.thumbnail(AMADEUS_LOGO);
    eb3 = eb3.thumbnail(AMADEUS_LOGO);
    eb1 = eb1.footer(CreateEmbedFooter::new(&footer));
    eb2 = eb2.footer(CreateEmbedFooter::new(&footer));
    eb3 = eb3.footer(CreateEmbedFooter::new(&footer));
    let mut max_apm = 0;
    if !flds.is_empty() {
      setm!{ fields1 = vec![]
           , fields2 = vec![]
           , fields3 = vec![] };
      for (kk, vv, mut papm) in flds {
        if vv.len() > 1 {
          fields1.push((kk.clone(), vv[0].clone(), true));
          fields2.push((kk.clone(), vv[1].clone(), true));
        }
        if !papm.len() > 1 {
          // drop last value of apm, because it's "not full"
          papm.truncate(papm.len() - 1);
          let max = papm.iter().max().unwrap_or(&0u64);
          max_apm = std::cmp::max(max_apm, *max);
          fields3.push(
            ( kk.clone()
            , papm.into_iter().enumerate().map(|(i, x)| (i as f32, x as f64))
            )
          );
        }
      }
      let mut apm_image: Option<String> = None;
      if !fields3.is_empty() {
        let (_, first_amp_list) = &fields3[0];
        let len: i32 = first_amp_list.len() as i32 - 1i32;
        { // because of Rc < > in BitMapBackend I need own scope here
          let root_area = BitMapBackend::new(&fname_apm, (1024, 768)).into_drawing_area();
          root_area.fill(&RGBColor(47, 49, 54))?; //2f3136
          let mut cc = ChartBuilder::on(&root_area)
            .margin(5u32)
            .set_all_label_area_size(50u32)
            .build_cartesian_2d(0.0f32..len as f32, 0.0f64..max_apm as f64)?;
          cc.configure_mesh()
            .label_style(("monospace", 16).into_font().color(&RGBColor(150, 150, 150)))
            .y_labels(10)
            .axis_style(RGBColor(80, 80, 80))
            .draw()?;
          let colors = gen_colors(fields3.len());
          for (i, (k, plx)) in fields3.into_iter().enumerate() {
            let (red, green, blue) = colors[i];
            let color = RGBColor(red, green, blue);
            let style = 
              if i < 4 { ShapeStyle::from(color) }
              else { color.stroke_width(1 + (i as u32 / 4)) };
            cc.draw_series(LineSeries::new(plx, style))?
              .label(&k)
              .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style));
          }
          cc.configure_series_labels()
            .position(SeriesLabelPosition::LowerRight)
            .border_style(BLACK)
            .label_font(("monospace", 19).into_font().color(&RGBColor(200, 200, 200)))
            .draw()?;
        }
        let a = CreateAttachment::path(std::path::Path::new(&fname_apm)).await?;
        match APM_PICS.send_message(&ctx, CreateMessage::new()
          .add_file(a)).await {
          Ok(msg) => {
            if !msg.attachments.is_empty() {
              let img_attachment = &msg.attachments[0];
              apm_image = Some(img_attachment.url.clone());
            }
          },
          Err(why) => {
            error!("Failed to download and post stream img {why}");
          }
        };
      }
      eb1 = eb1.fields(fields1);
      eb2 = eb2.fields(fields2);
      if let Some(apm) = apm_image {
        eb3 = eb3.image(&apm);
      }
    }
    let embeds = vec![ eb1, eb3, eb2 ];
    if let Ok(mut bot_msg) = msg.channel_id.send_message(&ctx, CreateMessage::new()
                                .embed( embeds[0].clone() )
                              ).await {
      let mut page: usize = 0;
      set!{ left  = ReactionType::Unicode(String::from("⬅️"))
          , right = ReactionType::Unicode(String::from("➡️")) };
      let _ = bot_msg.react(&ctx, left).await;
      let _ = bot_msg.react(&ctx, right).await;
      loop {
        let collector = bot_msg.reaction_collector(&ctx.shard)
                               .timeout(Duration::from_secs(360));
        if let Some(reaction) = collector.collect_single().await {
          let emoji = &reaction.emoji;
          match emoji.as_data().borrow() {
            "⬅️" => { 
              if page != 0 {
                page -= 1;
              }
            },
            "➡️" => { 
              if page != 2 {
                page += 1;
              }
            },
            _ => ()
          }
          if let Err(err) = bot_msg.edit(&ctx, EditMessage::default()
            .embed( embeds[page].clone() )
          ).await {
            error!("Shit happens {err}");
          }
          let _ = reaction.delete(&ctx).await;
        } else {
          let _ = bot_msg.delete_reactions(&ctx).await;
          break;
        };
      }
    } else {
      error!("Failed to post replay analyze data");
    }
    if let Err(why1) = fs::remove_file(&fname_apm).await {
      error!("Error removing apm png {why1}");
    }
    if let Err(why2) = fs::remove_file(&file.filename).await {
      error!("Error removing file: {why2}");
    }
  }
  Ok(())
}

pub async fn attach_replay( ctx: &Context
                          , msg: &Message
                          , file: &Attachment ) -> anyhow::Result<()> {
  if let Some(playa) = PLAYERS.iter().find(|p|
    !p.player.battletag.is_empty() && p.player.discord == msg.author.id.0.get()) {
    let battletag = playa.player.battletag.clone();
    if let Ok(bytes) = file.download().await {
      let mut fw3g = match File::create(&file.filename).await {
        Ok(replay) => replay,
        Err(why) => {
          error!("Error creating file: {:?}", why);
          channel_message(ctx, msg, "Error getting replay").await;
          return Err(anyhow!("Error creating file"));
        }
      };
      if let Err(why) = fw3g.write_all(&bytes).await {
        error!("Error writing to file: {why}");
        if let Err(why2) = fs::remove_file(&file.filename).await {
          error!("Error removing file: {why2}");
        }
        return Err(anyhow!("Error writing to file"));
      }
      let _ = fw3g.sync_data().await;
      let data_maybe = analyze(&file.filename, true).await;
      if let Err(why) = data_maybe {
        error!("Corrupted replay file? {why}");
        if let Err(why) = fs::remove_file(&file.filename).await {
          error!("Error removing file: {why}");
        }
        return Err(anyhow!("Corrupted replay file"));
      }
      let (_, flds) = data_maybe?;
      // only 2x2 and solo games
      if flds.len() == 2 || flds.len() == 4 {
        setm!{ found = false
             , max_apm = 0 };
        let mut fields1: Vec<(String, String)> = vec![];
        let mut fields2: Vec<(String, String, bool)> = vec![];
        let mut fields3 = vec![];
        for (btag, vv, mut papm) in flds {
          if battletag == btag {
            // so we see this player is indeed there
            found = true;
          }
          if !vv.is_empty() {
            let rqcl = {
              set!{ data = ctx.data.read().await
                  , rqcl = data.get::<ReqwestClient>().unwrap() };
              rqcl.clone()
            };
            let p_name =
              if let Some(aka) = check_aka(btag.as_str(), &rqcl).await {
                aka
              } else if btag.contains('#') {
                btag.split('#').collect::<Vec<&str>>()[0].to_string()
              } else {
                btag.clone()
              };
            fields1.push((p_name, vv[0].clone()));
          }
          if !papm.len() > 1 {
            // drop last value of apm, because it's "not full"
            papm.truncate(papm.len() - 1);
            let max = papm.iter().max().unwrap_or(&0);
            max_apm = std::cmp::max(max_apm, *max);
            fields3.push(
              ( btag.clone()
              , papm.into_iter().enumerate().map(|(i, x)| (i as f32, x as f64))
              )
            );
          }
        }
        if found {
          if let Some(guild_id) = msg.guild_id {
            // get last 15 games
            if let Ok(messages) = msg.channel_id.messages(&ctx,
              GetMessages::default().limit(15)
            ).await {
              for mmm in messages {
                if !mmm.embeds.is_empty()
                && !mmm.embeds[0].fields.is_empty()
                && mmm.attachments.is_empty() {
                  // start counting, we need two!
                  let mut same_count: u32 = 0;
                  for f in mmm.embeds[0].fields.clone() {
                    for (pf, _) in fields1.iter() {
                      if f.name == *pf {
                        same_count += 1;
                      }
                    }
                  }
                  // we've found some game which looks alike replay
                  if same_count > 1 {
                    for f in mmm.embeds[0].fields.clone() {
                      let mut modified = false;
                      for (pf, v) in fields1.iter() {
                        if f.name == *pf {
                          let mut already_inf_fields = false;
                          for (pff, vv, _) in fields2.iter() {
                            if pff == &f.name && vv.ends_with(v) {
                              already_inf_fields = true;
                              break;
                            }
                          }
                          modified = true;
                          if !already_inf_fields {
                            fields2.push((f.name.clone(), format!("{}\n{}", f.value, v), f.inline));
                            break;
                          }
                        }
                      }
                      if !modified {
                        fields2.push((f.name, f.value, f.inline));
                      }
                    }
                    let fname_apm = format!("{}_apm.png", &file.filename);
                    let mut apm_image: Option<String> = None;
                    if !fields3.is_empty() {
                      let (_, first_amp_list) = &fields3[0];
                      let len: i32 = first_amp_list.len() as i32 - 1i32;
                      { // because of Rc < > in BitMapBackend I need own scope here
                        let root_area = BitMapBackend::new(&fname_apm, (1024, 768)).into_drawing_area();
                        root_area.fill(&RGBColor(47, 49, 54))?; //2f3136
                        let mut cc = ChartBuilder::on(&root_area)
                          .margin(5u32)
                          .set_all_label_area_size(50u32)
                          .build_cartesian_2d(0.0f32..len as f32, 0.0f64..max_apm as f64)?;
                        cc.configure_mesh()
                          .label_style(("monospace", 16).into_font().color(&RGBColor(150, 150, 150)))
                          .y_labels(10)
                          .axis_style(RGBColor(80, 80, 80))
                          .draw()?;
                        let colors = gen_colors(fields3.len());
                        for (i, (k, plx)) in fields3.into_iter().enumerate() {
                          let (red, green, blue) = colors[i];
                          let color = RGBColor(red, green, blue);
                          let style = 
                            if i < 4 { ShapeStyle::from(color) }
                            else { color.stroke_width(1 + (i as u32 / 4)) };
                          cc.draw_series(LineSeries::new(plx, style))?
                            .label(&k)
                            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style));
                        }
                        cc.configure_series_labels()
                          .position(SeriesLabelPosition::LowerRight)
                          .border_style(BLACK)
                          .label_font(("monospace", 19).into_font().color(&RGBColor(200, 200, 200)))
                          .draw()?;
                      }
                      let a = CreateAttachment::path(std::path::Path::new(&fname_apm)).await?;
                      match APM_PICS.send_message(&ctx, CreateMessage::new()
                        .add_file(a)).await {
                        Ok(msg) => {
                          if !msg.attachments.is_empty() {
                            let img_attachment = &msg.attachments[0];
                            apm_image = Some(img_attachment.url.clone());
                          }
                        },
                        Err(why) => {
                          error!("Failed to download and post stream img {why}");
                        }
                      };
                    }

                    let nick = msg.author.nick_in(ctx, guild_id)
                                         .await
                                         .unwrap_or_else(|| msg.author.name.clone());
                    let mut e = CreateEmbed::new()
                      .title(&mmm.embeds[0].title.clone().unwrap())
                      .author(CreateEmbedAuthor::new(&nick).icon_url(&msg.author.face()))
                      .description(&mmm.embeds[0].description.clone().unwrap())
                      .footer(CreateEmbedFooter::new( mmm.embeds[0].footer.clone().unwrap().text ));
                    if !fields2.is_empty() {
                      e = e.fields(fields2);
                    }
                    if let Some(apm) = apm_image {
                      e = e.image(&apm);
                    }
                    if let Some(some_img) = mmm.embeds[0].image.clone() {
                      e = e.thumbnail(&some_img.url);
                    } else if let Some(hero) = mmm.embeds[0].thumbnail.clone() {
                      e = e.thumbnail(&hero.url);
                    }
                    if let Some(some_url) = &mmm.embeds[0].url {
                      e = e.url(some_url);
                    }
                    if let Some(colour) = &mmm.embeds[0].colour {
                      e = e.colour(colour.tuple());
                    }
                    if let Err(why) = msg.channel_id.send_message(ctx, CreateMessage::new()
                      .embed(e)
                    ).await {
                      error!("Failed to attach replay {why}");
                    } else {
                      // Success
                      if let Err(why) = mmm.delete(ctx).await {
                        error!("Failed to remove replaced message {why}");
                      }
                      if let Err(why) = fs::remove_file(&file.filename).await {
                        error!("Error removing file: {why}");
                      }
                      return Ok(());
                    }
                    break;
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  if let Err(why) = fs::remove_file(&file.filename).await {
    error!("Error removing file: {why}");
  }
  Err(anyhow!("Failed to attach replay"))
}

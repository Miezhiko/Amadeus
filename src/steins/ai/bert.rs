use crate::steins::ai::cache::{
  CACHE_ENG_STR,
  process_message_for_gpt
};

use rust_bert::pipelines::{
  conversation::{ ConversationManager
                , ConversationModel
                , ConversationConfig },
  question_answering::{ QaInput
                      , QuestionAnsweringModel
                      , QuestionAnsweringConfig },
  translation::{ Language
               , TranslationConfig
               , TranslationModel }
};

use tch::Device;
use tokio::{ task, sync::Mutex };
use once_cell::sync::Lazy;

use std::collections::HashMap;

use anyhow::Result;

use rand::{ seq::SliceRandom, Rng };

use super::neo::chat_neo;

static TRANSLATION_LIMIT: usize = 512;
static GPT_LIMIT: usize = 1000;

// models
static DEVICE: Lazy<Device> = Lazy::new(Device::cuda_if_available);

pub static EN2RUMODEL: Lazy<Mutex<TranslationModel>> =
  Lazy::new(||
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::EnglishToRussian, *DEVICE)
    ).unwrap()));

pub static RU2ENMODEL: Lazy<Mutex<TranslationModel>> =
  Lazy::new(||
    Mutex::new(TranslationModel::new(
      TranslationConfig::new(Language::RussianToEnglish, *DEVICE)
    ).unwrap()));

pub static QAMODEL: Lazy<Mutex<QuestionAnsweringModel>> =
  Lazy::new(||
    Mutex::new(QuestionAnsweringModel::new(
      QuestionAnsweringConfig {
        lower_case: false,
        device: *DEVICE,
        ..Default::default()
      }
    ).unwrap()));

pub static CONVMODEL: Lazy<Mutex<ConversationModel>> =
  Lazy::new(||
    Mutex::new(ConversationModel::new(
      ConversationConfig {
        min_length: 3,
        max_length: 64,
        min_length_for_response: 5,
        device: *DEVICE,
        ..Default::default()
      }
    ).unwrap()));

#[allow(clippy::type_complexity)]
pub static CHAT_CONTEXT: Lazy<Mutex<HashMap<u64, (ConversationManager, u32, u32)>>>
  = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn reinit() {
  let mut chat_context = CHAT_CONTEXT.lock().await;
  chat_context.clear();
}

pub async fn en2ru(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let en2ru_model = EN2RUMODEL.lock().await;
  task::spawn_blocking(move || {
    let mut something = text;
    if something.len() > TRANSLATION_LIMIT {
      if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
        something = something[i..].to_string();
      }
    }
    let output = en2ru_model.translate(&[something.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig EnglishToRussian");
      Ok(something)
    } else {
      Ok(output[0].clone())
    }
  }).await.unwrap()
}

pub async fn ru2en(text: String) -> Result<String> {
  if text.is_empty() {
    return Ok(String::new());
  }
  let ru2en_model = RU2ENMODEL.lock().await;
  task::spawn_blocking(move || {
    let mut something = text;
    if something.len() > TRANSLATION_LIMIT {
      if let Some((i, _)) = something.char_indices().rev().nth(TRANSLATION_LIMIT) {
        something = something[i..].to_string();
      }
    }
    let output = ru2en_model.translate(&[something.as_str()]);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(something)
    } else {
      let translation = &output[0];
      Ok(translation.clone())
    }
  }).await.unwrap()
}

// this is dangerous method!
pub async fn ru2en_many(texts: Vec<String>) -> Result<Vec<String>> {
  if texts.is_empty() {
    return Ok(vec![]);
  }
  let ru2en_model = EN2RUMODEL.lock().await;
  task::spawn_blocking(move || {
    let ttt = texts.iter().map(|t| t.as_str()).collect::<Vec<&str>>();
    let output = ru2en_model.translate(&ttt);
    if output.is_empty() {
      error!("Failed to translate with TranslationConfig RussianToEnglish");
      Ok(Vec::new())
    } else {
      Ok(output)
    }
  }).await.unwrap()
}

pub async fn ask(msg_content: String) -> Result<String> {
  info!("Generating GPT2 QA response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let qa_model = QAMODEL.lock().await;
  let mut question = process_message_for_gpt(&msg_content);
  if question.len() > GPT_LIMIT {
    if let Some((i, _)) = question.char_indices().rev().nth(GPT_LIMIT) {
      question = question[i..].to_string();
    }
  }
  let cache = 
    if cache_eng_vec.is_empty() {
      String::from("HUMBA")
    } else {
      cache_eng_vec
        .choose_multiple(&mut rand::thread_rng(), 100)
        .map(AsRef::as_ref).collect::<Vec<&str>>()
        .join(" ")
    };
  task::spawn_blocking(move || {
    let qa_input = QaInput {
      question, context: cache
    };
    // Get answer
    let answers = qa_model.predict(&[qa_input], 1, 32);
    if answers.is_empty() {
      error!("Failed to ansewer with QuestionAnsweringModel");
      Err(anyhow!("no output from GPT QA model"))
    } else {
      let my_answers = &answers[0];

      // we have several answers (hope they sorted by score)
      let answer = &my_answers[0];
      Ok(answer.answer.clone())
    }
  }).await.unwrap()
}

async fn chat_gpt2(something: String, user_id: u64) -> Result<String> {
  info!("Generating GPT2 response");
  let cache_eng_vec = CACHE_ENG_STR.lock().await;
  let conversation_model = CONVMODEL.lock().await;
  let mut chat_context = CHAT_CONTEXT.lock().await;
  task::spawn_blocking(move || {
    let output =
      if let Some((tracking_conversation, passed, x)) = chat_context.get_mut(&user_id) {
        if *x > 5 {
          chat_context.remove(&user_id);

          let mut conversation_manager = ConversationManager::new();
          let cache_slices = cache_eng_vec
                          .choose_multiple(&mut rand::thread_rng(), 64)
                          .map(AsRef::as_ref).collect::<Vec<&str>>();
          let encoded_history = conversation_model.encode_prompts(&cache_slices);
          let conv_id = conversation_manager.create(&something);
          if let Some(cm) = conversation_manager.get(&conv_id) {
            cm.load_from_history(cache_slices, encoded_history);
          }
          chat_context.insert( user_id
                            , ( conversation_manager, 0, 0 )
                            );
          if let Some(chat_cont) = chat_context.get_mut(&user_id) {
            let (registered_conversation, _, _) = chat_cont;
            conversation_model.generate_responses(registered_conversation)
          } else {
            return Err(anyhow!("Failed to cregister conversation for {}", &user_id));
          }
        } else {
          tracking_conversation.create(&something);
          *passed = 0; *x += 1;
          conversation_model.generate_responses(tracking_conversation)
        }
      } else {
        let mut conversation_manager = ConversationManager::new();
        let cache_slices = cache_eng_vec
                          .choose_multiple(&mut rand::thread_rng(), 10)
                          .map(AsRef::as_ref).collect::<Vec<&str>>();
        let encoded_history = conversation_model.encode_prompts(&cache_slices);
        let conv_id = conversation_manager.create(&something);
        if let Some(cm) = conversation_manager.get(&conv_id) {
          cm.load_from_history(cache_slices, encoded_history);
        }

        chat_context.insert( user_id
                            , ( conversation_manager, 0, 0 )
                            );

        if let Some(chat_cont) = chat_context.get_mut(&user_id) {
          let (registered_conversation, _, _) = chat_cont;
          conversation_model.generate_responses(registered_conversation)
        } else {
          return Err(anyhow!("Failed to cregister conversation for {}", &user_id));
        }
      };

    let out_values = output.values()
                            .cloned()
                            .map(str::to_string)
                            .collect::<Vec<String>>();

    if out_values.is_empty() {
      error!("Failed to chat with ConversationModel");
      Err(anyhow!("no output from GPT 2 model"))
    } else {
      Ok(out_values[0].clone())
    }
  }).await.unwrap()
}

pub async fn chat(something: String, user_id: u64) -> Result<String> {
  let rndx = rand::thread_rng().gen_range(0..4);
  let mut input = process_message_for_gpt(&something);
  if input.len() > GPT_LIMIT {
    if let Some((i, _)) = input.char_indices().rev().nth(GPT_LIMIT) {
      input = input[i..].to_string();
    }
  }
  if input.is_empty() {
    return Err(anyhow!("empty input"));
  }
  match rndx {
    0 => chat_neo(input).await,
    _ => chat_gpt2(input, user_id).await
  }
}

#[cfg(test)]
mod bert_tests {
  use super::*;
  #[test]
  #[ignore]
  fn chat_big_text_test() {
    let conversation_model = ConversationModel::new(
        ConversationConfig {
            min_length: 3,
            max_length: 64,
            min_length_for_response: 5,
            ..Default::default()
          }
        ).unwrap();

    let mut conversation_manager = ConversationManager::new();

    let input_1 = "вот например вчера я думал, стоит ли заказать пиццу. Но иногда я себе говорю \"Блин, хватит жрать пиццу, есть же и другая еда, скушай там салатик или супчик, надоела пицца эта\" ну и я такой подумал. Надо монетку подбросить. Но у меня все монетки в копилке. Сейчас даже фото этой копилки найду. И я установил на телефон приложение, которое подбрасывают монетку. Вот. И я говорю себе. \"ну если выпадет решка - я заказываю пиццу, если орёл - заказываю супчик\" ну и я как бы уже не делаю это случайным, это ведь я выбрал, кто будет орлом, а кто решкой. По идее я должен был подкинуть монетку изначально на то, кто будет орлом, а кто будет решкой. Поэтому, наверное, это уже не была случайность. Тем не менее. Подбрасываю я такой эту монетку в телефоне, выпадает решка. И я расстраиваюсь, потому что я уже не хочу пиццу, я уже настроился на супчик, уже нашёл, где заказать хороший тай суп, поэтому я просто говорю себе \"нет, приложение херня, скачаю другое, это какое-то забагованное\", качаю другое, и там падает орёл. И я довольный заказываю себе суп и сырные палочки. И вот это ведь уже не случайность ? Я думаю, совсем не случайность. Тем не менее, я немного успокоил свою совесть и дал таким образом себе подумать над тем, а что я действительно хочу заказать ? И это вот такой банальный пример. Но можно ли вообще считать, что есть какая-то случайность ? Например ставлю я себе \"случайная раса\" в варкрафте и мне падают там 8 из 10 игр эльфы условные. Это  всё ещё случайность ? Или это уже поломанный скрипт ? Было бы ли правильно, если бы в \"случайности\" 3 раза из 12 игр падала каждая раса ? Я так не думаю, это уже не случайность, а статистика. Но я всё ещё верю, что любые случайности не очень случайны, хотя случаю стоит очень доверять, многие свои решения в жизни я сделал грубо говоря \"подбрасывая монетку\", и они оказались очень правильными. Хотя в конечном итоге не решка или орёл решали, буду ли я счастлив. Не так ли? Что ты думаешь по этому поводу?";
    let input_2 = "дорогая! У меня к тебе вопрос. Ты когда-то была в хвойном лесу? Знаешь, когда ты идёшь в лес, находишь хорошую такую берёзу, разбиваешь ей кору, вставляешь такую специальную штуку и там начинает по капельке капать бирюзовый сок. Ты на ночь оставляешь там банку, а в 5 утра тебя будит дедушка, вы идёте ранней осенью по заваленному листьями лесу, собираете грибочки в корзину, которую бабушка дала и так прохладно, в лесу ведь всегда холодно ? Ты пособирала эти грибочки час, устала и вы идёте к той берёзке, берёшь эту баночку, там ещё что-то попадало, какие-то листики, хвоя, маленькие веточки и делаешь такой глоток этого холодного берёзового сока и у тебя такое единение с природой, ты прям чувствуешь как будто дерево пьёшь, да ? Не то, что эти берёзовые соки в магазинах.. Химия сплошная, не передаёт совсем этот вкус природы и этот прекрасный настрой раннего осеннего леса с лёгкой усталостью. Это не заменит не красивая упаковка, не удобство обслуживания на кассе. Я правда никогда не пил берёзовый сок, да и дедушки у меня не было, и в лес я не люблю ходить. Но я об этом много слышал. И читал рассказы. Тот же Чехов. Как ты относишься к книгам Чехова ? Я знаю, что сейчас популярно считать, что он переоценен. Средний сатирик, Фёдор Михайлович намного лучше и всё вот это. Что ж, может и так, но может вся суть Чехова как раз и есть в этой детской простоте, наивности и хорошем выводе о прекрасных вещах, по типу классического \"В человеке должно быть всё прекрасно, и душа и тело, и одежда и мысли\" . Не уверен, что правильно эту цитату привёл, но в младшей школе классная руководительница постоянно говорила мне её, когда видела мои изрисованные в непонятно чём тетради и мой корявый почерк. Но что-то мы отходим от темы. Ты когда-то бывала в лесу ?";

    let translation_config =
      TranslationConfig::new(Language::RussianToEnglish, Device::cuda_if_available());

    let model = TranslationModel::new(translation_config).unwrap();

    let en_1 = model.translate(&[input_1]);
    let en_2 = model.translate(&[input_2]);

    assert!( !en_1.is_empty() );
    assert!( !en_2.is_empty() );

    let cache_slices = vec![en_1[0].as_str()];

    let encoded_history = conversation_model.encode_prompts(&cache_slices);

    let conv_id = conversation_manager.create(en_2[0].as_str());
    if let Some(cm) = conversation_manager.get(&conv_id) {
      cm.load_from_history(cache_slices, encoded_history);
    }

    let output = conversation_model.generate_responses(&mut conversation_manager);

    let out_values = output.values()
                            .cloned()
                            .map(str::to_string)
                            .collect::<Vec<String>>();

    assert!( !out_values.is_empty() )
  }
}

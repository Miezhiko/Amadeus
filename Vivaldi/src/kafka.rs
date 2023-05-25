use mozart::bert::chat::chat_gpt2;

use futures::stream::FuturesUnordered;
use futures::{ StreamExt, TryStreamExt };

use log::{ info, warn, error };

use rdkafka::{
  config::ClientConfig,
  consumer::stream_consumer::StreamConsumer,
  consumer::Consumer,
  message::OwnedMessage,
  producer::{ FutureProducer, FutureRecord },
  Message
};

use async_recursion::async_recursion;

use crate::{
  gpt4free,
  opengpt
};

async fn record_owned_message_receipt(msg: &OwnedMessage) {
  // Like `record_borrowed_message_receipt`, but takes an `OwnedMessage`
  // instead, as in a real-world use case  an `OwnedMessage` might be more
  // convenient than a `BorrowedMessage`.
  info!("Message received: {}", msg.offset());
}

#[async_recursion]
pub async fn chat_gpt2_kafka(msg: u64
                           , chan: u64
                           , something: String
                           , user_id: u64
                           , russian: bool
                           , gtry: u32) -> anyhow::Result<(String, String)> {
  if gtry > 0 {
    warn!("GPT2: trying again: {gtry}");
  }
  match chat_gpt2(something.clone(), user_id, true).await {
    Ok(result) => {
      let k_key = format!("{chan}|{user_id}|{msg}");
      Ok((k_key, result))
    }, Err(why) => {
      error!("GPT2: Failed to generate response: {why}");
      if gtry > 9 {
        error!("GPT2: failed to generate response 10 times!");
        Err( why )
      } else {
        chat_gpt2_kafka(msg, chan, something, user_id, russian, gtry + 1).await
      }
    }
  }
}

async fn mozart_process<'a>(msg: OwnedMessage) -> Option<(String, String)> {
  info!("Generating response for Kafka message {}", msg.offset());
  match msg.payload() {
    Some(payload_bytes) => {
      let payload = std::str::from_utf8(payload_bytes).expect("Kafka: payload is not UTF8");
      let key = msg.key().expect("Kafka: no key proviced!");
      let key_str = std::str::from_utf8(&key).expect("Kafka: key is not string!");
      let key3 = key_str.split('|')
                        .filter(|&x| !x.is_empty())
                        .collect::<Vec<&str>>();
      if key3.len() < 3 {
        error!("Error: Invalid key split");
        return None;
      }

      let chan      = key3[0].parse::<u64>().unwrap();
      let user_id   = key3[1].parse::<u64>().unwrap();
      let msg       = key3[2].parse::<u64>().unwrap();
      let k_key     = format!("{chan}|{user_id}|{msg}");

      let mut fmode = true;
      if payload.contains("please") || payload.contains("пожалуйста") {
        fmode = false;
      } else if payload.contains("Please")
             || payload.contains("Пожалуйста")
             || payload.contains("PLEASE") {
        if let Ok(gpt4free_result) = opengpt::chatbase::generate( payload ) {
          return Some((k_key, gpt4free_result));
        }
        fmode = false;
      }

      if let Ok(gpt4free_result)        = gpt4free::useless::generate( payload, fmode ).await {
        Some((k_key, gpt4free_result))
      } else if let Ok(gpt4free_result) = gpt4free::deepai::generate( payload, fmode ).await {
        Some((k_key, gpt4free_result))
      } else if let Ok(gpt4free_result) = gpt4free::gptworldAi::generate( payload, fmode ).await {
        Some((k_key, gpt4free_result))
      } else if let Ok(gpt4free_result) = gpt4free::italygpt::generate( payload, fmode ).await {
        Some((k_key, gpt4free_result))
      } else if let Ok(gpt4free_result) = opengpt::chatbase::generate( payload ) {
        Some((k_key, gpt4free_result))
      } else if let Ok(gpt4free_result) = gpt4free::theb::generate( payload ) {
        Some((k_key, gpt4free_result))
      } else {
        let gpt2gen =
        chat_gpt2_kafka( key3[2].parse::<u64>().unwrap_or(0)
                       , key3[0].parse::<u64>().unwrap()
                       , payload.to_string()
                       , key3[1].parse::<u64>().unwrap()
                       , false // TODO: check for russian
                       , 0 ).await;
        match gpt2gen {
          Ok(response) => Some(response),
          Err(err) => {
            error!("Failed to generate gpt stuff on Kafka {err}");
            None
          }
        }
      }
    },
    None => None
  }
}

async fn run_async_processor(
  brokers: String,
  group_id: String,
  input_topic: String,
  output_topic: String,
) {
  // Create the `StreamConsumer`
  // to receive the messages from the topic in form of a `Stream`.
  let consumer: StreamConsumer = ClientConfig::new()
      .set("group.id", &group_id)
      .set("bootstrap.servers", &brokers)
      .set("enable.partition.eof", "false")
      .set("session.timeout.ms", "6000")
      .set("enable.auto.commit", "false")
      .create()
      .expect("Consumer creation failed");

  consumer
    .subscribe(&[&input_topic])
    .expect("Can't subscribe to specified topic");

  let producer: FutureProducer = ClientConfig::new()
    .set("bootstrap.servers", &brokers)
    .set("message.timeout.ms", "5000")
    .create()
    .expect("Producer creation error");

  let stream_processor = consumer.stream().try_for_each(|borrowed_message| {
    let producer = producer.clone();
    let output_topic = output_topic.to_string();
    async move {
      // Process each message
      // Borrowed messages can't outlive the consumer they are received from, so they need to
      // be owned in order to be sent to a separate thread.
      let owned_message = borrowed_message.detach();
      record_owned_message_receipt(&owned_message).await;
      tokio::spawn(async move {
        if let Some((k_key, response)) = mozart_process(owned_message).await {
          let produce_future = producer.send(
            FutureRecord::to(&output_topic)
              .key(&k_key)
              .payload(&response),
            std::time::Duration::from_secs(0)
          );
          match produce_future.await {
            Ok(delivery)  => println!("Kafka response sent: {:?}", delivery),
            Err((e, _))   => println!("Error on kafka response: {:?}", e)
          }
        }
      });
      Ok(())
    }
  });

  info!("Starting kafka event loop");
  stream_processor.await.expect("Kafka stream processing failed");
  info!("Kafka stream processing terminated");
}

pub fn run_with_workers(num_workers: u32) {
  let _ = (0..num_workers).map(|_| {
    tokio::spawn(run_async_processor(
      "localhost:9092".to_owned(),
      "kalmarity_group".to_owned(),
      "Salieri".to_owned(),
      "Kalmarity".to_owned(),
    ))
  }).collect::<FuturesUnordered<_>>()
    .for_each(|_| async { () });
}

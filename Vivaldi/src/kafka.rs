use futures::stream::FuturesUnordered;
use futures::{ StreamExt, TryStreamExt };

use log::{ info, error };

use rdkafka::{
  config::ClientConfig,
  consumer::stream_consumer::StreamConsumer,
  consumer::Consumer,
  message::OwnedMessage,
  producer::{ FutureProducer, FutureRecord },
  Message
};

async fn record_owned_message_receipt(msg: &OwnedMessage) {
  info!("Message received: {}", msg.offset());
}

async fn gpt_process(msg: OwnedMessage) -> Option<(String, String)> {
  info!("Generating response for Kafka message {}", msg.offset());
  match msg.payload() {
    Some(payload_bytes) => {
      let payload = std::str::from_utf8(payload_bytes).expect("Kafka: payload is not UTF8");
      let key     = msg.key().expect("Kafka: no key proviced!");
      let key_str = std::str::from_utf8(key).expect("Kafka: key is not string!");
      let key3    = key_str.split('|')
                           .filter(|&x| !x.is_empty())
                           .collect::<Vec<&str>>();

      if key3.len() < 3 {
        error!("Error: Invalid key split");
        return None;
      }

      /* 
      let chan      = key3[0].parse::<u64>().unwrap_or(0);
      let user_id   = key3[1].parse::<u64>().unwrap_or(0);
      let msg       = key3[2].parse::<u64>().unwrap_or(0);
      let k_key     = format!("{chan}|{user_id}|{msg}");
      */

      if let Ok(chat_result) = chat::chat(payload, "Kalmarity").await {
        Some((key_str.to_owned(), chat_result))
      } else { None }
    }, None => None
  }
}

async fn run_async_processor(
  brokers: String,
  group_id: String,
  input_topic: String,
  output_topic: String,
) {
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
      let owned_message = borrowed_message.detach();
      record_owned_message_receipt(&owned_message).await;
      tokio::spawn(async move {
        if let Some((k_key, response)) = gpt_process(owned_message).await {
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
  std::mem::drop( (0..num_workers).map(|_| {
    tokio::spawn(run_async_processor(
      "localhost:9092".to_owned(),
      "kalmarity_group".to_owned(),
      "Salieri".to_owned(),
      "Kalmarity".to_owned(),
    ))
  }).collect::<FuturesUnordered<_>>()
    .for_each(|_| async {}) );
}

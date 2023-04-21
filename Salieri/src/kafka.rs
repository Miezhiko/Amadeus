use mozart::bert::{
  chat::chat_gpt2_send,
  LUKASHENKO
};

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
  // Like `record_borrowed_message_receipt`, but takes an `OwnedMessage`
  // instead, as in a real-world use case  an `OwnedMessage` might be more
  // convenient than a `BorrowedMessage`.
  info!("Message received: {}", msg.offset());
}

async fn mozart_process<'a>(msg: OwnedMessage) -> String {
  info!("Starting expensive computation on message {}", msg.offset());
  info!(
    "Expensive computation completed on message {}",
    msg.offset()
  );
  match msg.payload_view::<str>() {
    Some(Ok(payload)) => {
      let key = msg.key().expect("Kafka: no key proviced!");
      let key_str = std::str::from_utf8(&key).expect("Kafka: key is not string!");
      let key3 = key_str.split('|')
                        .filter(|&x| !x.is_empty())
                        .collect::<Vec<&str>>();
      if key3.len() < 3 {
        return "Error: Invalid key split".to_owned();
      }

      if let Err(why) = chat_gpt2_send( Some( key3[2].parse::<u64>().unwrap() )
                                      , key3[0].parse::<u64>().unwrap()
                                      , payload.to_string()
                                      , key3[1].parse::<u64>().unwrap()
                                      , false
                                      , false // TODO: check for russian
                                      , 0 ).await {
        error!("Failed to generate response, {why}");
      } else {
        info!("GPT2 response sent to {LUKASHENKO}!");
      }
      format!("Payload len for {} is {}", payload, payload.len())
    },
    Some(Err(_)) => "Error: Message payload is not a string".to_owned(),
    None => "Error: No payload".to_owned()
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
        let computation_result = mozart_process(owned_message).await;
        let produce_future = producer.send(
          FutureRecord::to(&output_topic)
            .key("some key")
            .payload(&computation_result),
          std::time::Duration::from_secs(0)
        );
        match produce_future.await {
          Ok(delivery) => println!("Sent: {:?}", delivery),
          Err((e, _)) => println!("Error: {:?}", e)
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

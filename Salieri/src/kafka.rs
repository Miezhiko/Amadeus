use std::thread;
use std::time::Duration;

use futures::stream::FuturesUnordered;
use futures::{StreamExt, TryStreamExt};

use log::info;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::message::{BorrowedMessage, OwnedMessage};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::Message;

async fn record_borrowed_message_receipt(msg: &BorrowedMessage<'_>) {
  // Simulate some work that must be done in the same order as messages are
  // received; i.e., before truly parallel processing can begin.
  info!("Message received: {}", msg.offset());
}

async fn record_owned_message_receipt(_msg: &OwnedMessage) {
  // Like `record_borrowed_message_receipt`, but takes an `OwnedMessage`
  // instead, as in a real-world use case  an `OwnedMessage` might be more
  // convenient than a `BorrowedMessage`.
}

// Emulates an expensive, synchronous computation.
fn expensive_computation<'a>(msg: OwnedMessage) -> String {
  info!("Starting expensive computation on message {}", msg.offset());
  thread::sleep(Duration::from_millis(rand::random::<u64>() % 5000));
  info!(
    "Expensive computation completed on message {}",
    msg.offset()
  );
  match msg.payload_view::<str>() {
    Some(Ok(payload)) => format!("Payload len for {} is {}", payload, payload.len()),
    Some(Err(_)) => "Message payload is not a string".to_owned(),
    None => "No payload".to_owned()
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
      record_borrowed_message_receipt(&borrowed_message).await;
      // Borrowed messages can't outlive the consumer they are received from, so they need to
      // be owned in order to be sent to a separate thread.
      let owned_message = borrowed_message.detach();
      record_owned_message_receipt(&owned_message).await;
      tokio::spawn(async move {
        // The body of this block will be executed on the main thread pool,
        // but we perform `expensive_computation` on a separate thread pool
        // for CPU-intensive tasks via `tokio::task::spawn_blocking`.
        let computation_result =
            tokio::task::spawn_blocking(|| expensive_computation(owned_message))
                .await
                .expect("failed to wait for expensive computation");
        let produce_future = producer.send(
            FutureRecord::to(&output_topic)
                .key("some key")
                .payload(&computation_result),
            Duration::from_secs(0),
        );
        match produce_future.await {
            Ok(delivery) => println!("Sent: {:?}", delivery),
            Err((e, _)) => println!("Error: {:?}", e),
        }
      });
      Ok(())
    }
  });
}

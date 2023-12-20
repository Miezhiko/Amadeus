use crate::{
  naoko::options::IOptions,
  common::msg::reply
};

use futures::{
  stream::FuturesUnordered,
  StreamExt, TryStreamExt
};

use log::{ info, error };

use rdkafka::{
  config::ClientConfig,
  consumer::stream_consumer::StreamConsumer,
  consumer::Consumer,
  message::OwnedMessage,
  producer::{ FutureProducer, FutureRecord },
  Message
};

use serenity::{
  prelude::*,
  builder::CreateMessage,
  model::id::{ ChannelId, MessageId }
};

async fn record_owned_message_receipt(msg: &OwnedMessage) {
  info!("Message received: {}", msg.offset());
}

async fn got_message(msg: OwnedMessage, ctx: &Arc<Context>) -> anyhow::Result<()> {
  info!("Got message from kafka {}", msg.offset());
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

      let chan_id   = key3[0].parse::<u64>().unwrap_or(0);
      let msg_id    = key3[2].parse::<u64>().unwrap_or(0);

      let chan: ChannelId = ChannelId::new(chan_id);

      if msg_id != 0 {
        if let Ok(mmsg) = chan.message(ctx, MessageId::new(*msg_id)).await {
          reply(ctx, &mmsg, &payload).await;
        }
      } else {
        chan.send_message(ctx, CreateMessage::new().content(&payload)).await?;
      }
      Ok(())
    }, None => Err(anyhow!("kafka sent empty payload"))
  }
}

async fn run_async_processor(
  brokers: String,
  group_id: String,
  input_topic: String,
  ctx: &Arc<Context>
) {
  let consumer: StreamConsumer = ClientConfig::new()
      .set("group.id", &group_id)
      .set("bootstrap.servers", &brokers)
      .set("enable.partition.eof", "false")
      .set("session.timeout.ms", "5000")
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
    async move {
      let owned_message = borrowed_message.detach();
      record_owned_message_receipt(&owned_message).await;
      tokio::spawn(async move {
        if let Err(why) = got_message(owned_message, ctx).await {
          error!("can't process kafka msg: ${why}");
        }
      });
      Ok(())
    }
  });

  info!("Starting kafka event loop");
  stream_processor.await
                  .expect("Kafka stream processing failed");
  info!("Kafka stream processing terminated");
}

pub fn run_with_workers(num_workers: u32, opts: IOptions, ctx: &Arc<Context>) {
  std::mem::drop( (0..num_workers).map(|_| {
    tokio::spawn(run_async_processor(
      opts.kafka_address.clone(),
      opts.kafka_group.clone(),
      opts.kafka_sink.clone(),
      ctx
    ))
  }).collect::<FuturesUnordered<_>>()
    .for_each(|_| async {}) );
}

async fn request(opts: IOptions, args: &options::Args, text: &str) {
  let producer: &FutureProducer = &ClientConfig::new()
    .set("bootstrap.servers", opts.kafka_address)
    .set("message.timeout.ms", "5000")
    .create()
    .expect("Producer creation error");
  let _delivery_status = producer
    .send(
      FutureRecord::to(opts.kafka_target)
        .payload(text)
        .key( &format!("{}|{}|{}"
            , args.channel
            , args.user
            , args.msg)),
      Duration::from_secs(0),
    ).await;
}

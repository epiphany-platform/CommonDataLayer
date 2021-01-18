/// We are using tokio::sync::broadcast to support multiple connections via WebSocket.
/// The idea is, that if two clients ask for the same stream of data, you don't wanna query it twice.
/// Instead you listen on different task (See: `tokio::spawn` in `EventSubscriber::new`) and then send message to broadcast channel.
/// Each websocket client has its own Receiver.
/// Thanks to that we are not only reusing connection, but also limit dangerous `consumer.leak()` usage.
use crate::config::KafkaConfig;
use anyhow::Context as _Context;
use futures::task::{Context as FutCtx, Poll};
use futures::{Stream, StreamExt, TryStreamExt};
use juniper::FieldResult;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    error::KafkaError,
    ClientConfig, Message,
};
use std::pin::Pin;
use tokio::sync::broadcast;

// TODO: Probably could be replaced by OwnedMessage from kafkard?
/// Owned generic message received from kafka.
#[derive(Clone, Debug)]
pub struct KafkaEvent {
    pub key: Option<String>,
    pub payload: Option<String>,
}

/// Wrapper to prevent accidental sending data to channel. `Sender` is used only for subscription mechanism
pub struct EventSubscriber(broadcast::Sender<Result<KafkaEvent, KafkaError>>);

// We are using Box<dyn> approach (recommended) by Tokio maintainers,
// as unfortunately `broadcast::Receiver` doesn't implement `Stream` trait,
// and it is hard to achieve it without major refactor. Therefore we are using `async_stream` as a loophole.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = FieldResult<KafkaEvent>> + Send + Sync>>,
}

impl EventSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub fn new(config: &KafkaConfig, topic: &str) -> Result<(Self, EventStream), anyhow::Error> {
        let (tx, rx) = broadcast::channel(32);

        log::debug!("Create new consumer for topic: {}", topic);

        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &config.group_id)
            .set("bootstrap.servers", &config.brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        rdkafka::consumer::Consumer::subscribe(&consumer, &[topic])
            .context("Can't subscribe to specified topics")?;

        let sink = tx.clone();

        let consumer = Box::leak(Box::new(consumer));
        tokio::spawn(async move {
            let stream = consumer.start().map_ok(move |msg| {
                let key = msg
                    .key()
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .map(|s| s.to_string());
                let payload = msg
                    .payload()
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .map(|s| s.to_string());
                KafkaEvent { key, payload }
            });

            tokio::pin!(stream);

            while let Some(item) = stream.next().await {
                sink.send(item).ok();
            }

            log::warn!("Kafka stream has ended");
        });

        Ok((Self(tx), EventStream::new(rx)))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> EventStream {
        EventStream::new(self.0.subscribe())
    }
}

impl EventStream {
    fn new(mut rx: broadcast::Receiver<Result<KafkaEvent, KafkaError>>) -> Self {
        let stream = async_stream::try_stream! {
            loop {
                let item = rx.recv().await??;
                yield item;
            }
        };
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for EventStream {
    type Item = FieldResult<KafkaEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut FutCtx<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

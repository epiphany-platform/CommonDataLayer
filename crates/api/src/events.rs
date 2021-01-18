/// We are using tokio::sync::broadcast to support multiple connections via WebSocket.
/// The idea is, that if two clients ask for the same stream of data, you don't wanna query it twice.
/// Instead you listen on different task (See: `tokio::spawn` in `EventSubscriber::new`) and then send message to broadcast channel.
/// Each websocket client has its own Receiver.
/// Thanks to that we are not only reusing connection, but also limit dangerous `consumer.leak()` usage.
use crate::config::KafkaConfig;
use futures::task::{Context as FutCtx, Poll};
use futures::{Future, Stream, StreamExt, TryStreamExt};
use juniper::FieldResult;
use std::pin::Pin;
use tokio::sync::broadcast;
use utils::messaging_system::consumer::CommonConsumer;
use utils::messaging_system::Error;

// TODO: Rename it to generic `Event` after adding support to RabbitMQ
/// Owned generic message received from kafka.
#[derive(Clone, Debug)]
pub struct KafkaEvent {
    pub key: Option<String>,
    pub payload: Option<String>,
}

/// Wrapper to prevent accidental sending data to channel. `Sender` is used only for subscription mechanism
pub struct EventSubscriber(broadcast::Sender<Result<KafkaEvent, Error>>);

// We are using Box<dyn> approach (recommended) by Tokio maintainers,
// as unfortunately `broadcast::Receiver` doesn't implement `Stream` trait,
// and it is hard to achieve it without major refactor. Therefore we are using `async_stream` as a loophole.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = FieldResult<KafkaEvent>> + Send + Sync>>,
}

impl EventSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub async fn new<F, Fut>(
        config: &KafkaConfig,
        topic: &str,
        on_close: F,
    ) -> Result<(Self, EventStream), anyhow::Error>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: Future<Output = ()>,
    {
        let (tx, rx) = broadcast::channel(32);

        log::debug!("Create new consumer for topic: {}", topic);

        let consumer =
            CommonConsumer::new_kafka(&config.group_id, &config.brokers, &[topic]).await?;
        let consumer = Box::leak(Box::new(consumer));

        let sink = tx.clone();
        let topic = String::from(topic);
        tokio::spawn(async move {
            let stream = consumer.consume().await.map_ok(move |msg| {
                let key = msg.key().map(|s| s.to_string()).ok();
                let payload = msg.payload().map(|s| s.to_string()).ok();
                KafkaEvent { key, payload }
            });

            tokio::pin!(stream);

            while let Some(item) = stream.next().await {
                sink.send(item).ok();
            }

            on_close(topic);
        });

        Ok((Self(tx), EventStream::new(rx)))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> EventStream {
        EventStream::new(self.0.subscribe())
    }
}

impl EventStream {
    fn new(mut rx: broadcast::Receiver<Result<KafkaEvent, Error>>) -> Self {
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

/// We are using tokio::sync::broadcast to support multiple connections via WebSocket.
/// The idea is, that if two clients ask for the same stream of data, you don't wanna query it twice.
/// Instead you listen on different task (See: `tokio::spawn` in `EventSubscriber::new`) and then send message to broadcast channel.
/// Each websocket client has its own Receiver.
/// Thanks to that we are not only reusing connection, but also limit dangerous `consumer.leak()` usage.
use crate::config::CommunicationMethodConfig;
use async_trait::async_trait;
use futures::task::{Context as FutCtx, Poll};
use futures::{Future, Stream};
use juniper::FieldResult;
use std::pin::Pin;
use tokio::sync::broadcast::{self, Sender};
use utils::communication::{
    consumer::{CommonConsumer, CommonConsumerConfig, ConsumerHandler},
    message::CommunicationMessage,
};

/// Owned generic message received from message queue.
#[derive(Clone, Debug)]
pub struct Event {
    pub key: Option<String>,
    pub payload: Option<String>,
}

/// Wrapper to prevent accidental sending data to channel. `Sender` is used only for subscription mechanism
pub struct EventSubscriber(Sender<Event>);

// We are using Box<dyn> approach (recommended) by Tokio maintainers,
// as unfortunately `broadcast::Receiver` doesn't implement `Stream` trait,
// and it is hard to achieve it without major refactor. Therefore we are using `async_stream` as a loophole.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = FieldResult<Event>> + Send + Sync>>,
}

struct Handler {
    sink: Sender<Event>,
}

#[async_trait]
impl ConsumerHandler for Handler {
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let key = msg.key().map(|s| s.to_string()).ok();
        let payload = msg.payload().map(|s| s.to_string()).ok();
        let event = Event { key, payload };
        self.sink.send(event).ok();
        Ok(())
    }
}

impl EventSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub async fn new<F, Fut>(
        config: CommunicationMethodConfig,
        source: &str,
        on_close: F,
    ) -> Result<(Self, EventStream), anyhow::Error>
    where
        F: FnOnce(String) -> Fut + Send + 'static,
        Fut: Future<Output = ()>,
    {
        let (tx, rx) = broadcast::channel(32);

        tracing::debug!("Create new consumer for: {}", source);

        let config = match &config {
            CommunicationMethodConfig::Kafka { group_id, brokers } => CommonConsumerConfig::Kafka {
                group_id: &group_id,
                brokers: &brokers,
                topic: source,
            },
            CommunicationMethodConfig::Amqp {
                connection_string,
                consumer_tag,
            } => CommonConsumerConfig::Amqp {
                connection_string: &connection_string,
                consumer_tag: &consumer_tag,
                queue_name: source,
                options: None,
            },
            CommunicationMethodConfig::Grpc => {
                anyhow::bail!("GRPC communication method does not support event subscription")
            }
        };

        let consumer = CommonConsumer::new(config).await?;

        let sink = tx.clone();
        let source = String::from(source);
        tokio::spawn(async move {
            consumer.run(Handler { sink }).await.ok();

            on_close(source);
        });

        Ok((Self(tx), EventStream::new(rx)))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> EventStream {
        EventStream::new(self.0.subscribe())
    }
}

impl EventStream {
    fn new(mut rx: broadcast::Receiver<Event>) -> Self {
        let stream = async_stream::try_stream! {
            loop {
                let item = rx.recv().await?;
                yield item;
            }
        };
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for EventStream {
    type Item = FieldResult<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut FutCtx<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

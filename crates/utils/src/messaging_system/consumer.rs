use anyhow::Context;
use async_stream::try_stream;
use futures_util::stream::{Stream, StreamExt};
pub use lapin::options::BasicConsumeOptions;
use lapin::types::FieldTable;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    ClientConfig,
};
use std::sync::Arc;
use tokio_amqp::LapinTokioExt;

use super::{
    message::CommunicationMessage, message::KafkaCommunicationMessage,
    message::RabbitCommunicationMessage, Result,
};

pub enum CommonConsumerConfig<'a> {
    Kafka(KafkaConsumerConfig<'a>),
    Amqp(AmqpConsumerConfig<'a>),
}

pub struct KafkaConsumerConfig<'a> {
    pub brokers: &'a str,
    pub group_id: &'a str,
    pub topic: &'a str,
}

pub struct AmqpConsumerConfig<'a> {
    pub connection_string: &'a str,
    pub consumer_tag: &'a str,
    pub queue_name: &'a str,
    pub options: Option<BasicConsumeOptions>,
}

pub enum CommonConsumer {
    Kafka {
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    },
    Amqp {
        consumer: lapin::Consumer,
    },
}
impl CommonConsumer {
    pub async fn new(config: CommonConsumerConfig<'_>) -> Result<Self> {
        match config {
            CommonConsumerConfig::Kafka(kafka) => {
                Self::new_kafka(kafka.group_id, kafka.brokers, &[kafka.topic]).await
            }
            CommonConsumerConfig::Amqp(amqp) => {
                Self::new_amqp(
                    amqp.connection_string,
                    amqp.consumer_tag,
                    amqp.queue_name,
                    amqp.options,
                )
                .await
            }
        }
    }

    async fn new_kafka(group_id: &str, brokers: &str, topics: &[&str]) -> Result<Self> {
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        rdkafka::consumer::Consumer::subscribe(&consumer, topics)
            .context("Can't subscribe to specified topics")?;

        Ok(CommonConsumer::Kafka {
            consumer: Arc::new(consumer),
        })
    }

    async fn new_amqp(
        connection_string: &str,
        consumer_tag: &str,
        queue_name: &str,
        consume_options: Option<BasicConsumeOptions>,
    ) -> Result<Self> {
        let consume_options = consume_options.unwrap_or_default();
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;
        let consumer = channel
            .basic_consume(
                queue_name,
                consumer_tag,
                consume_options,
                FieldTable::default(),
            )
            .await?;
        Ok(CommonConsumer::Amqp { consumer })
    }

    pub async fn consume(
        &mut self,
    ) -> impl Stream<Item = Result<Box<dyn CommunicationMessage + '_>>> {
        try_stream! {
        match self {
            CommonConsumer::Kafka { consumer } => {
                let mut message_stream = consumer.start();
                    while let Some(message) = message_stream.next().await {
                        let message = message?;
                        yield Box::new(KafkaCommunicationMessage{message,consumer:consumer.clone()}) as Box<dyn CommunicationMessage>;
                    }
                }
                CommonConsumer::Amqp {
                    consumer,
                } => {
                    while let Some(message) = consumer.next().await {
                        let message = message?;
                        yield Box::new(RabbitCommunicationMessage{channel:message.0, delivery:message.1})as Box<dyn CommunicationMessage>;
                    }
                }
            }
        }
    }

    /// Leaks consumer to guarantee consumer never be dropped.
    /// Static consumer lifetime is required for consumed messages to be passed to spawned futures.
    ///
    /// Use with causion as it can cause memory leaks.
    pub fn leak(self) -> &'static mut CommonConsumer {
        Box::leak(Box::new(self))
    }
}

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
    kafka_ack_queue::KafkaAckQueue, message::AmqpCommunicationMessage,
    message::CommunicationMessage, message::KafkaCommunicationMessage, Result,
};

pub enum CommonConsumerConfig<'a> {
    Kafka {
        brokers: &'a str,
        group_id: &'a str,
        topic: &'a str,
    },
    Amqp {
        connection_string: &'a str,
        consumer_tag: &'a str,
        queue_name: &'a str,
        options: Option<BasicConsumeOptions>,
    },
}
pub enum CommonConsumer {
    Kafka {
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
        ack_queue: Arc<KafkaAckQueue>,
    },
    Amqp {
        consumer: lapin::Consumer,
    },
}
impl CommonConsumer {
    pub async fn new(config: CommonConsumerConfig<'_>) -> Result<Self> {
        match config {
            CommonConsumerConfig::Kafka {
                group_id,
                brokers,
                topic,
            } => Self::new_kafka(group_id, brokers, &[topic]).await,
            CommonConsumerConfig::Amqp {
                connection_string,
                consumer_tag,
                queue_name,
                options,
            } => Self::new_amqp(connection_string, consumer_tag, queue_name, options).await,
        }
    }

    async fn new_kafka(group_id: &str, brokers: &str, topics: &[&str]) -> Result<Self> {
        let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("enable.auto.offset.store", "false")
            .set("auto.offset.reset", "earliest")
            .create()
            .context("Consumer creation failed")?;

        rdkafka::consumer::Consumer::subscribe(&consumer, topics)
            .context("Can't subscribe to specified topics")?;

        Ok(CommonConsumer::Kafka {
            consumer: Arc::new(consumer),
            ack_queue: Default::default(),
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
                CommonConsumer::Kafka { consumer, ack_queue} => {
                    let mut message_stream = consumer.start();
                    while let Some(message) = message_stream.next().await {
                        let message = message?;
                        ack_queue.add(&message);
                        yield Box::new(KafkaCommunicationMessage{message,consumer:consumer.clone(),ack_queue:ack_queue.clone()}) as Box<dyn CommunicationMessage>;
                    }
                }
                CommonConsumer::Amqp {
                    consumer,
                } => {
                    while let Some(message) = consumer.next().await {
                        let message = message?;
                        yield Box::new(AmqpCommunicationMessage{channel:message.0, delivery:message.1})as Box<dyn CommunicationMessage>;
                    }
                }
            }
        }
    }

    /// Leaks consumer to guarantee consumer never be dropped.
    /// Static consumer lifetime is required for consumed messages to be passed to spawned futures.
    ///
    /// Use with caution as it can cause memory leaks.
    pub fn leak(self) -> &'static mut CommonConsumer {
        Box::leak(Box::new(self))
    }
}

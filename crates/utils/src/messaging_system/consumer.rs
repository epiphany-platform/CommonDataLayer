use anyhow::Context;
use async_stream::try_stream;
use futures_util::stream::{Stream, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable};
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

pub enum CommonConsumer {
    Kafka {
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    },
    RabbitMq {
        consumer: lapin::Consumer,
    },
}
impl CommonConsumer {
    pub async fn new_kafka(
        group_id: &str,
        brokers: &str,
        topics: &[&str],
    ) -> Result<CommonConsumer> {
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

    pub async fn new_rabbit(
        connection_string: &str,
        consumer_tag: &str,
        queue_name: &str,
    ) -> Result<CommonConsumer> {
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
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(CommonConsumer::RabbitMq { consumer })
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
                CommonConsumer::RabbitMq {
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

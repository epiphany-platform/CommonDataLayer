use lapin::{options::BasicPublishOptions, BasicProperties, Channel, Connection};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::time::Duration;
use tokio_amqp::LapinTokioExt;

use super::CommunicationResult;

pub enum CommonPublisher {
    Kafka {
        producer: FutureProducer,
    },
    RabbitMq {
        _connection: Box<Connection>,
        channel: Channel,
    },
}
impl CommonPublisher {
    pub async fn new_rabbit(connection_string: &str) -> CommunicationResult<CommonPublisher> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;

        Ok(CommonPublisher::RabbitMq {
            _connection: Box::new(connection),
            channel,
        })
    }

    pub async fn new_kafka(brokers: &str) -> CommunicationResult<CommonPublisher> {
        let publisher = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .set("compression.type", "none")
            .set("max.in.flight.requests.per.connection", "1")
            .create()?;
        Ok(CommonPublisher::Kafka {
            producer: publisher,
        })
    }

    pub async fn publish_message(
        &self,
        topic_or_exchange: &str,
        key: &str,
        payload: Vec<u8>,
    ) -> CommunicationResult<()> {
        match self {
            CommonPublisher::Kafka { producer } => {
                let delivery_status = producer.send(
                    FutureRecord::to(topic_or_exchange)
                        .payload(&payload)
                        .key(key),
                    Duration::from_secs(5),
                );
                delivery_status.await.map_err(|x| x.0)?;
                Ok(())
            }
            CommonPublisher::RabbitMq {
                _connection,
                channel,
            } => {
                channel
                    .basic_publish(
                        topic_or_exchange,
                        key,
                        BasicPublishOptions::default(),
                        payload,
                        BasicProperties::default(),
                    )
                    .await?
                    .await?;
                Ok(())
            }
        }
    }
}

use lapin::{options::BasicPublishOptions, BasicProperties, Channel};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use std::time::Duration;
use tokio_amqp::LapinTokioExt;

use super::Result;

#[derive(Clone)]
pub enum CommonPublisher {
    Kafka { producer: FutureProducer },
    RabbitMq { channel: Channel },
}
impl CommonPublisher {
    pub async fn new_rabbit(connection_string: &str) -> Result<CommonPublisher> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;

        Ok(CommonPublisher::RabbitMq { channel })
    }

    pub async fn new_kafka(brokers: &str) -> Result<CommonPublisher> {
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
    ) -> Result<()> {
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
            CommonPublisher::RabbitMq { channel } => {
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

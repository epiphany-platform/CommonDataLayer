use lapin::{options::BasicPublishOptions, BasicProperties, Channel};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use reqwest::Client;
use std::time::Duration;
use tokio_amqp::LapinTokioExt;
use url::Url;

use super::{Error, Result};

#[derive(Clone)]
pub enum CommonPublisher {
    Kafka { producer: FutureProducer },
    Amqp { channel: Channel },
    Rest { url: Url, client: Client },
    Grpc { service: &'static str },
}
impl CommonPublisher {
    pub async fn new_amqp(connection_string: &str) -> Result<Self> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;

        Ok(Self::Amqp { channel })
    }

    pub async fn new_kafka(brokers: &str) -> Result<Self> {
        let publisher = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .set("compression.type", "none")
            .set("max.in.flight.requests.per.connection", "1")
            .create()?;
        Ok(Self::Kafka {
            producer: publisher,
        })
    }

    pub async fn new_rest(url: Url) -> Result<Self> {
        Ok(Self::Rest {
            url,
            client: reqwest::Client::new(),
        })
    }

    pub async fn new_grpc(service: &'static str) -> Result<Self> {
        Ok(Self::Grpc { service })
    }

    pub async fn publish_message(
        &self,
        destination: &str,
        key: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
        match self {
            CommonPublisher::Kafka { producer } => {
                let delivery_status = producer.send(
                    FutureRecord::to(destination).payload(&payload).key(key),
                    Duration::from_secs(5),
                );
                delivery_status.await.map_err(|x| x.0)?;
                Ok(())
            }
            CommonPublisher::Amqp { channel } => {
                channel
                    .basic_publish(
                        destination,
                        key,
                        BasicPublishOptions::default(),
                        payload,
                        BasicProperties::default().with_delivery_mode(2), // persistent messages
                    )
                    .await?
                    .await?;
                Ok(())
            }
            CommonPublisher::Rest { url, client } => {
                let url = url.join(&format!("{}/{}", destination, key)).unwrap();

                client.post(url).body(payload).send().await?;

                Ok(())
            }
            CommonPublisher::Grpc { service } => {
                let addr = destination.into();
                let mut client = rpc::generic::connect(addr, service).await?;
                let response = client
                    .handle(rpc::generic::Message {
                        key: key.into(),
                        payload,
                    })
                    .await;

                match response {
                    Ok(_) => Ok(()),
                    Err(status) => Err(Error::GrpcStatusCode(status.code().description().into())),
                }
            }
        }
    }
}

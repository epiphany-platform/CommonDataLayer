#![allow(unused_imports, unused_variables)]
#[cfg(feature = "amqp")]
use lapin::{options::BasicPublishOptions, BasicProperties, Channel};
#[cfg(feature = "kafka")]
use rdkafka::{
    message::OwnedHeaders,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
#[cfg(feature = "http")]
use reqwest::Client;
use std::time::Duration;
#[cfg(feature = "amqp")]
use tokio_amqp::LapinTokioExt;
#[cfg(feature = "http")]
use tracing_utils::http::RequestBuilderTracingExt;
use url::Url;

use super::{Error, Result};

#[derive(Clone)]
pub enum CommonPublisher {
    #[cfg(feature = "kafka")]
    Kafka { producer: FutureProducer },
    #[cfg(feature = "amqp")]
    Amqp { channel: Channel },
    #[cfg(feature = "http")]
    Rest { url: Url, client: Client },
    #[cfg(feature = "grpc")]
    Grpc,
}
impl CommonPublisher {
    #[cfg(feature = "amqp")]
    pub async fn new_amqp(connection_string: &str) -> Result<Self> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await?;
        let channel = connection.create_channel().await?;

        Ok(Self::Amqp { channel })
    }

    #[cfg(feature = "kafka")]
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

    #[cfg(feature = "http")]
    pub async fn new_rest(url: Url) -> Result<Self> {
        Ok(Self::Rest {
            url,
            client: reqwest::Client::new(),
        })
    }

    #[cfg(feature = "grpc")]
    pub async fn new_grpc() -> Result<Self> {
        Ok(Self::Grpc)
    }

    pub async fn publish_message(
        &self,
        destination: &str,
        key: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "kafka")]
            CommonPublisher::Kafka { producer } => {
                let delivery_status = producer.send(
                    FutureRecord::to(destination)
                        .payload(&payload)
                        .key(key)
                        .headers(tracing_utils::kafka::inject_span(OwnedHeaders::new())),
                    Duration::from_secs(5),
                );
                delivery_status.await.map_err(|x| x.0)?;
                Ok(())
            }
            #[cfg(feature = "amqp")]
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
            #[cfg(feature = "http")]
            CommonPublisher::Rest { url, client } => {
                let url = url.join(&format!("{}/{}", destination, key)).unwrap();

                client.post(url).body(payload).inject_span().send().await?;

                Ok(())
            }
            #[cfg(feature = "grpc")]
            CommonPublisher::Grpc => {
                let addr = destination.into();
                let mut client = rpc::generic::connect(addr).await?;
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

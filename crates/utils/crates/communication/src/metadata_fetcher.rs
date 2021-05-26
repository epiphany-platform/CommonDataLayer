#![allow(unused_imports, unused_variables)]
use std::time::Duration;

use anyhow::Context;
#[cfg(feature = "kafka")]
use rdkafka::producer::{BaseProducer, Producer};
#[cfg(feature = "kafka")]
use rdkafka::ClientConfig;
#[cfg(feature = "amqp")]
use tokio_amqp::LapinTokioExt;

use super::Result;

pub enum MetadataFetcher {
    #[cfg(feature = "kafka")]
    Kafka { producer: BaseProducer },
    #[cfg(feature = "amqp")]
    Amqp { connection: lapin::Connection },
    #[cfg(feature = "grpc")]
    Grpc,
}

impl MetadataFetcher {
    #[cfg(feature = "kafka")]
    pub async fn new_kafka(brokers: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()
            .context("Metadata fetcher creation failed")?;

        Ok(Self::Kafka { producer })
    }

    #[cfg(feature = "amqp")]
    pub async fn new_amqp(connection_string: &str) -> Result<Self> {
        let connection = lapin::Connection::connect(
            connection_string,
            lapin::ConnectionProperties::default().with_tokio(),
        )
        .await
        .context("Metadata fetcher creation failed")?;

        Ok(Self::Amqp { connection })
    }

    #[cfg(feature = "grpc")]
    pub fn new_grpc() -> Result<Self> {
        Ok(Self::Grpc)
    }

    pub async fn destination_exists(&self, destination: &str) -> Result<bool> {
        let owned_destination = String::from(destination);

        match self {
            #[cfg(feature = "amqp")]
            MetadataFetcher::Amqp { connection } => {
                let channel: lapin::Channel = connection
                    .create_channel()
                    .await
                    .context("Metadata fetcher AMQP channel creation failed")?;
                let result = channel
                    .exchange_declare(
                        destination,
                        lapin::ExchangeKind::Topic,
                        lapin::options::ExchangeDeclareOptions {
                            passive: true,
                            ..Default::default()
                        },
                        Default::default(),
                    )
                    .await;

                match result {
                    Err(lapin::Error::ProtocolError(amqp_error)) => {
                        if let lapin::protocol::AMQPErrorKind::Soft(
                            lapin::protocol::AMQPSoftError::NOTFOUND,
                        ) = amqp_error.kind()
                        {
                            Ok(false)
                        } else {
                            Err(lapin::Error::ProtocolError(amqp_error).into())
                        }
                    }
                    Err(e) => Err(e.into()),
                    Ok(()) => Ok(true),
                }
            }
            #[cfg(feature = "kafka")]
            MetadataFetcher::Kafka { producer } => {
                let producer = producer.clone();
                let metadata = tokio::task::spawn_blocking(move || {
                    let client = producer.client();
                    client.fetch_metadata(Some(&owned_destination), Duration::from_secs(5))
                })
                .await??;

                Ok(metadata
                    .topics()
                    .iter()
                    .any(|topic| topic.name() == destination))
            }
            #[cfg(feature = "grpc")]
            MetadataFetcher::Grpc => {
                let client = rpc::generic::connect(owned_destination).await;
                Ok(client.is_ok())
            }
        }
    }
}

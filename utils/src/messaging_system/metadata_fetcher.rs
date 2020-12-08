use std::time::Duration;

use anyhow::Context;
use rdkafka::{producer::BaseProducer, ClientConfig};

use super::CommunicationResult;

pub enum MetadataFetcher {
    Kafka { producer: BaseProducer },
}

impl MetadataFetcher {
    pub async fn new_kafka(brokers: &str) -> CommunicationResult<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .create()
            .context("Metadata fetcher creation failed")?;

        Ok(Self::Kafka { producer })
    }

    pub async fn topic_exists(&self, topic: &str) -> CommunicationResult<bool> {
        match self {
            Self::Kafka { producer } => {
                let owned_topic = String::from(topic);
                let producer = producer.clone();

                let metadata = tokio::task::spawn_blocking(move || {
                    let client = producer.client();
                    client.fetch_metadata(Some(&owned_topic), Duration::from_secs(5))
                })
                .await??;

                Ok(metadata
                    .topics()
                    .iter()
                    .map(|topic| topic.name())
                    .any(|name| name == topic))
            }
        }
    }
}

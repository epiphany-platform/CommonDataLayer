use std::time::Duration;

use anyhow::Context;
use rdkafka::{producer::BaseProducer, ClientConfig};

use super::Result;

pub struct KafkaMetadataFetcher {
    producer: BaseProducer,
}

impl KafkaMetadataFetcher {
    pub async fn new(brokers: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .create()
            .context("Metadata fetcher creation failed")?;

        Ok(Self { producer })
    }

    pub async fn topic_exists(&self, topic: &str) -> Result<bool> {
        let owned_topic = String::from(topic);
        let producer = self.producer.clone();

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

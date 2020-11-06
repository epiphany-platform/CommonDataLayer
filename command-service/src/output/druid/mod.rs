use crate::communication::resolution::Resolution;
use crate::communication::GenericMessage;
pub use crate::output::druid::config::DruidOutputConfig;
pub use crate::output::druid::error::Error;
use crate::output::error::OutputError;
use crate::output::OutputPlugin;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::{sync::Arc, time::Duration};
use utils::metrics::counter;

mod config;
mod error;

pub struct DruidOutputPlugin {
    producer: FutureProducer,
    topic: Arc<String>,
}

impl DruidOutputPlugin {
    pub async fn new(args: DruidOutputConfig) -> Result<Self, Error> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", &args.brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .map_err(Error::ProducerCreation)?,
            topic: Arc::new(args.topic),
        })
    }

    async fn store_message(
        producer: FutureProducer,
        msg: GenericMessage,
        topic: &str,
    ) -> Resolution {
        let key = msg.object_id.to_string();
        let record = FutureRecord {
            topic: &topic,
            partition: None,
            payload: Some(&msg.payload),
            key: Some(&key),
            timestamp: Some(msg.timestamp),
            headers: None,
        };

        match producer.send(record, Duration::from_secs(0)).await {
            Err((err, _)) => Resolution::StorageLayerFailure {
                description: err.to_string(),
            },
            Ok(_) => {
                counter!("cdl.command-service.store.druid", 1);

                Resolution::Success
            }
        }
    }
}

#[async_trait::async_trait]
impl OutputPlugin for DruidOutputPlugin {
    async fn handle_message(&self, msg: GenericMessage) -> Result<Resolution, OutputError> {
        let producer = self.producer.clone();
        let topic = Arc::clone(&self.topic);

        let resolution = DruidOutputPlugin::store_message(producer, msg, topic.as_str()).await;

        Ok(resolution)
    }

    fn name(&self) -> &'static str {
        "Druid timeseries"
    }
}

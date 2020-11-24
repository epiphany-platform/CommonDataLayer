use crate::communication::resolution::Resolution;
use crate::communication::GenericMessage;
pub use crate::output::druid::config::DruidOutputConfig;
pub use crate::output::druid::error::Error;
use crate::output::OutputPlugin;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use utils::metrics::counter;

mod config;
mod error;

pub struct DruidOutputPlugin {
    producer: FutureProducer,
    topic: String,
}

impl DruidOutputPlugin {
    pub async fn new(args: DruidOutputConfig) -> Result<Self, Error> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", &args.brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .map_err(Error::ProducerCreation)?,
            topic: args.topic,
        })
    }
}

#[async_trait::async_trait]
impl OutputPlugin for DruidOutputPlugin {
    async fn handle_message(&self, msg: GenericMessage) -> Resolution {
        let key = msg.object_id.to_string();
        let record = FutureRecord {
            topic: &self.topic,
            partition: None,
            payload: Some(&msg.payload),
            key: Some(&key),
            timestamp: Some(msg.timestamp),
            headers: None,
        };

        match self.producer.send(record, Duration::from_secs(0)).await {
            Err((err, _)) => Resolution::StorageLayerFailure {
                description: err.to_string(),
            },
            Ok(_) => {
                counter!("cdl.command-service.store.druid", 1);

                Resolution::Success
            }
        }
    }

    fn name(&self) -> &'static str {
        "Druid timeseries"
    }
}

use crate::communication::resolution::Resolution;
pub use crate::output::druid::config::DruidOutputConfig;
pub use crate::output::druid::error::Error;
use crate::output::OutputPlugin;
use futures::stream::{self, StreamExt};
use log::error;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::Value;
use std::time::Duration;
use utils::message_types::BorrowedInsertMessage;
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
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let key = msg.object_id.to_string();
        let payloads = match deserialize_payloads(&msg) {
            Ok(result) => result,
            Err(err) => return err,
        };
        let messages: Vec<FutureRecord<_, _>> = payloads
            .iter()
            .map(|payload| FutureRecord {
                topic: &self.topic,
                partition: None,
                payload: Some(payload),
                key: Some(&key),
                timestamp: Some(msg.timestamp),
                headers: None,
            })
            .collect();
        stream::iter(messages)
            .then(|message| async {
                match self.producer.send(message, Duration::from_secs(0)).await {
                    Err((err, _)) => Err(Resolution::StorageLayerFailure {
                        description: err.to_string(),
                    }),
                    Ok(_) => {
                        counter!("cdl.command-service.store.druid", 1);
                        Ok(Resolution::Success)
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;
        Resolution::Success
    }

    fn name(&self) -> &'static str {
        "Druid timeseries"
    }
}

fn deserialize_payloads(msg: &BorrowedInsertMessage<'_>) -> Result<Vec<Vec<u8>>, Resolution> {
    let result: Result<Vec<Value>, serde_json::Error> =
        serde_json::from_str(&msg.data.get().to_string());
    match result {
        Ok(values) => Ok(values
            .into_iter()
            .map(|payload: Value| serde_json::to_vec(&payload).unwrap())
            .collect()),
        Err(err) => {
            let context = msg.data.to_string();
            error!(
                "Failed to read payload, cause `{}`, context `{}`",
                err, context
            );
            Err(Resolution::UserFailure {
                description: err.to_string(),
                context,
            })
        }
    }
}

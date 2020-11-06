use crate::communication::GenericMessage;
use crate::report::{Error, ReportServiceInstance};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

const APPLICATION_NAME: &str = "Command Service";

pub struct VerboseReportServiceConfig {
    pub producer: FutureProducer,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
}

pub struct VerboseReportServiceInstance {
    pub producer: FutureProducer,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
    pub msg: GenericMessage,
}

impl VerboseReportServiceConfig {
    pub fn new(brokers: String, topic: String, output_plugin: String) -> Result<Self, Error> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("message.timeout.ms", "5000")
                .create()
                .map_err(Error::ProducerCreation)?,
            topic: Arc::new(topic),
            output_plugin: Arc::new(output_plugin),
        })
    }
}

#[async_trait::async_trait]
impl ReportServiceInstance for VerboseReportServiceInstance {
    async fn report(&self, description: &str) -> Result<(), Error> {
        let payload = json!({
            "application": APPLICATION_NAME,
            "output_plugin": self.output_plugin,
            "description": description,
            "object_id": self.msg.object_id,
            "payload": String::from_utf8_lossy(&self.msg.payload)
        })
        .to_string();

        let record = FutureRecord {
            topic: &self.topic,
            partition: None,
            payload: Some(&payload),
            key: Some("command_service.status"),
            timestamp: None,
            headers: None,
        };

        self.producer
            .send(record, Duration::from_secs(0))
            .await
            .map_err(|err| Error::FailedToReport(err.0))?;

        Ok(())
    }
}

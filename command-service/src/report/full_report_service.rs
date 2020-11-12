use crate::communication::GenericMessage;
use crate::report::{Error, ReportServiceInstance};
use serde_json::json;
use std::sync::Arc;
use utils::messaging_system::publisher::CommonPublisher;

const APPLICATION_NAME: &str = "Command Service";

pub struct FullReportServiceConfig {
    pub producer: CommonPublisher,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
}

pub struct FullReportServiceInstance {
    pub producer: CommonPublisher,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
    pub msg: GenericMessage,
}

impl FullReportServiceConfig {
    pub async fn new(brokers: String, topic: String, output_plugin: String) -> Result<Self, Error> {
        Ok(Self {
            producer: CommonPublisher::new_kafka(&brokers)
                .await
                .map_err(Error::ProducerCreation)?,
            topic: Arc::new(topic),
            output_plugin: Arc::new(output_plugin),
        })
    }
}

#[async_trait::async_trait]
impl ReportServiceInstance for FullReportServiceInstance {
    async fn report(&mut self, description: &str) -> Result<(), Error> {
        let payload = json!({
            "application": APPLICATION_NAME,
            "output_plugin": self.output_plugin.as_str(),
            "description": description,
            "object_id": self.msg.object_id,
            "payload": String::from_utf8_lossy(&self.msg.payload)
        })
        .to_string();

        self.producer
            .publish_message(self.topic.as_str(), "command_service.status", payload.into_bytes())
            .await
            .map_err(Error::FailedToReport)?;

        Ok(())
    }
}

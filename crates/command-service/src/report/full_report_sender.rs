use crate::report::{Error, Reporter};
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use utils::message_types::OwnedInsertMessage;
use utils::messaging_system::publisher::CommonPublisher;
use uuid::Uuid;

const APPLICATION_NAME: &str = "Command Service";

#[derive(Clone)]
pub struct FullReportSenderBase {
    pub producer: CommonPublisher,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
}

pub struct FullReportSender {
    pub producer: CommonPublisher,
    pub topic: Arc<String>,
    pub output_plugin: Arc<String>,
    pub msg: OwnedInsertMessage,
}

#[derive(Serialize)]
struct ReportBody<'a> {
    application: &'static str,
    output_plugin: &'a str,
    description: &'a str,
    object_id: Uuid,
    payload: Value,
}

impl FullReportSenderBase {
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
impl Reporter for FullReportSender {
    async fn report(self: Box<Self>, description: &str) -> Result<(), Error> {
        let payload = ReportBody {
            application: APPLICATION_NAME,
            output_plugin: self.output_plugin.as_str(),
            description,
            object_id: self.msg.object_id,
            payload: self.msg.data,
        };

        self.producer
            .publish_message(
                self.topic.as_str(),
                "command_service.status",
                serde_json::to_vec(&payload).map_err(Error::FailedToProduceErrorMessage)?,
            )
            .await
            .map_err(Error::FailedToReport)
    }
}

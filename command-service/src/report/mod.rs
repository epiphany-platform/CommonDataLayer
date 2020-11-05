pub use config::ReportServiceConfig;
pub use error::Error;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::time::Duration;
use uuid::Uuid;

const APPLICATION_NAME: &str = "Command Service";

mod config;
mod error;

pub struct ReportService {
    producer: FutureProducer,
    topic: String,
}

impl ReportService {
    pub fn new(args: ReportServiceConfig) -> Result<Self, Error> {
        Ok(Self {
            producer: ClientConfig::new()
                .set("bootstrap.servers", &args.broker)
                .set("message.timeout.ms", "5000")
                .create()
                .map_err(Error::ProducerCreation)?,
            topic: args.topic,
        })
    }

    pub async fn report_failure(
        &self,
        output_plugin: &str,
        description: &str,
        object_id: Uuid,
    ) -> Result<(), Error> {
        let payload = format!(
            r#"{{"application": "{}", "output_plugin": "{}", "description": "{}", "object_id": "{}" }}"#,
            APPLICATION_NAME,
            output_plugin,
            description.replace('"', r#"\""#),
            object_id,
        );

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

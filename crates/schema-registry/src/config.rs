use serde::Deserialize;
use std::path::PathBuf;
use utils::communication::metadata_fetcher::MetadataFetcher;
use utils::settings::*;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub input_port: u16,
    pub import_file: Option<PathBuf>,
    pub export_dir: Option<PathBuf>,

    pub postgres: PostgresSettings,

    pub kafka: Option<PublisherKafkaSettings>,
    pub amqp: Option<AmqpSettings>,

    pub monitoring: MonitoringSettings,
}

impl Settings {
    pub async fn metadata_fetcher(&self) -> anyhow::Result<MetadataFetcher> {
        Ok(match (&self.kafka, &self.amqp, self.communication_method) {
            (Some(kafka), _, CommunicationMethod::Kafka) => {
                MetadataFetcher::new_kafka(kafka.brokers.as_str()).await?
            }
            (_, Some(amqp), CommunicationMethod::Amqp) => {
                MetadataFetcher::new_amqp(amqp.exchange_url.as_str()).await?
            }
            (_, _, CommunicationMethod::GRpc) => {
                MetadataFetcher::new_grpc()?
            }
            _ => todo!(),
        })
    }
}

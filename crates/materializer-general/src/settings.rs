use communication_utils::publisher::CommonPublisher;
use serde::Deserialize;
use settings_utils::{LogSettings, MonitoringSettings, PostgresSettings};
use utils::notification::NotificationSettings;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input_port: u16,
    pub cache_capacity: usize,

    pub postgres: PostgresSettings,
    pub kafka: Option<KafkaProducerSettings>,
    #[serde(default)]
    pub notifications: NotificationSettings,

    pub monitoring: MonitoringSettings,

    pub log: LogSettings,

    pub services: ServicesSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServicesSettings {
    pub schema_registry_url: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaProducerSettings {
    pub brokers: String,
}

impl Settings {
    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        if let Some(kafka) = &self.kafka {
            Ok(CommonPublisher::new_kafka(&kafka.brokers).await?)
        } else {
            anyhow::bail!("Missing Kafka config for notifications")
        }
    }
}

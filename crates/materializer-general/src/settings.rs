use communication_utils::publisher::CommonPublisher;
use serde::Deserialize;
use settings_utils::{LogSettings, MonitoringSettings, PostgresSettings};
use utils::notification::NotificationSettings;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input_port: u16,

    pub postgres: PostgresSettings,
    pub kafka: KafkaProducerSettings,
    #[serde(default)]
    pub notifications: NotificationSettings,

    pub monitoring: MonitoringSettings,

    pub log: LogSettings,
}

#[derive(Debug, Deserialize)]
pub struct KafkaProducerSettings {
    pub brokers: String,
}

impl Settings {
    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        Ok(CommonPublisher::new_kafka(&self.kafka.brokers).await?)
    }
}

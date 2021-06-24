use communication_utils::publisher::CommonPublisher;
use serde::Deserialize;
use settings_utils::*;
use utils::notification::NotificationSettings;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,

    pub postgres: PostgresSettings,
    pub input_port: u16,

    pub kafka: Option<ConsumerKafkaSettings>,
    pub amqp: Option<AmqpSettings>,

    #[serde(default)]
    pub notifications: NotificationSettings,

    pub monitoring: MonitoringSettings,

    #[serde(default)]
    pub log: LogSettings,
}

impl Settings {
    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        publisher(
            self.kafka.as_ref().map(|kafka| kafka.brokers.as_str()),
            self.amqp.as_ref().map(|amqp| amqp.exchange_url.as_str()),
            None,
        )
        .await
    }
}

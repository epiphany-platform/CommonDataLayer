use serde::{Deserialize, Serialize};

use utils::settings::{
    AmqpSettings, ConsumerKafkaSettings, GRpcSettings, LogSettings, MonitoringSettings,
};
use utils::{
    communication::{parallel_consumer::ParallelCommonConsumer, publisher::CommonPublisher},
    task_limiter::TaskLimiter,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub cache_capacity: usize,
    #[serde(default = "default_async_task_limit")]
    pub async_task_limit: usize,

    pub kafka: Option<ConsumerKafkaSettings>,
    pub amqp: Option<AmqpSettings>,
    pub grpc: Option<GRpcSettings>,

    pub monitoring: MonitoringSettings,

    pub services: ServicesSettings,

    pub log: LogSettings,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMethod {
    Kafka,
    Amqp,
    #[serde(rename = "grpc")]
    GRpc,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ServicesSettings {
    pub schema_registry_url: String,
}

const fn default_async_task_limit() -> usize {
    32
}

impl Settings {
    pub async fn consumer(&self) -> anyhow::Result<ParallelCommonConsumer> {
        match (
            &self.kafka,
            &self.amqp,
            &self.grpc,
            &self.communication_method,
        ) {
            (Some(kafka), _, _, CommunicationMethod::Kafka) => {
                kafka
                    .parallel_consumer(TaskLimiter::new(self.async_task_limit))
                    .await
            }
            (_, Some(amqp), _, CommunicationMethod::Amqp) => {
                amqp.parallel_consumer(TaskLimiter::new(self.async_task_limit))
                    .await
            }
            (_, _, Some(grpc), CommunicationMethod::GRpc) => grpc.parallel_consumer().await,
            _ => anyhow::bail!("Unsupported consumer specification"),
        }
    }

    pub async fn producer(&self) -> anyhow::Result<CommonPublisher> {
        Ok(
            match (
                &self.kafka,
                &self.amqp,
                &self.grpc,
                &self.communication_method,
            ) {
                (Some(kafka), _, _, CommunicationMethod::Kafka) => {
                    CommonPublisher::new_kafka(&kafka.brokers).await?
                }
                (_, Some(amqp), _, CommunicationMethod::Amqp) => {
                    CommonPublisher::new_amqp(&amqp.exchange_url).await?
                }
                (_, _, Some(_), CommunicationMethod::GRpc) => CommonPublisher::new_grpc().await?,
                _ => anyhow::bail!("Unsupported consumer specification"),
            },
        )
    }
}

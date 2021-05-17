use anyhow::bail;
use serde::{Deserialize, Serialize};
use utils::communication::consumer::BasicConsumeOptions;
use utils::communication::parallel_consumer::{
    ParallelCommonConsumer, ParallelCommonConsumerConfig,
};
use utils::communication::publisher::CommonPublisher;
use utils::settings::*;
use utils::task_limiter::TaskLimiter;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub communication_method: CommunicationMethod,
    pub repository_kind: RepositoryKind,

    #[serde(default = "default_async_task_limit")]
    pub async_task_limit: usize,

    // Repository settings - based on repository_kind
    pub postgres: Option<PostgresSettings>,
    pub victoria_metrics: Option<VictoriaMetricsSettings>,
    pub druid: Option<DruidSettings>,

    // Communication settings - based on communication_method
    pub kafka: Option<KafkaSettings>,
    pub amqp: Option<AmqpSettings>,
    pub grpc: Option<GRpcSettings>,

    #[serde(default)]
    pub notifications: NotificationSettings,

    pub listener: ListenerSettings,

    pub monitoring: MonitoringSettings,

    #[serde(default)]
    pub log: LogSettings,
}

const fn default_async_task_limit() -> usize {
    32
}

#[derive(Debug, Deserialize)]
pub struct ListenerSettings {
    #[serde(default)]
    pub ordered_sources: String,
    #[serde(default)]
    pub unordered_sources: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryKind {
    Postgres,
    VictoriaMetrics,
    Druid,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DruidSettings {
    pub topic: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct KafkaSettings {
    pub brokers: String,
    pub group_id: String,
}

#[derive(Deserialize, Debug)]
pub struct AmqpSettings {
    pub exchange_url: String,
    pub tag: String,
    pub consume_options: Option<BasicConsumeOptions>,
}

impl ListenerSettings {
    fn is_empty(&self) -> bool {
        self.ordered_sources.is_empty() && self.unordered_sources.is_empty()
    }
}

impl Settings {
    pub async fn consumers(
        &self,
        task_limit: usize,
    ) -> anyhow::Result<Vec<ParallelCommonConsumer>> {
        let task_limiter = TaskLimiter::new(task_limit);

        match (&self.kafka, &self.amqp, &self.grpc) {
            (Some(kafka), _, _) if self.communication_method == CommunicationMethod::Kafka => {
                if self.listener.is_empty() {
                    bail!("Missing list of listener queues")
                }

                kafka
                    .parallel_consumers(
                        self.listener
                            .ordered_sources
                            .split(',')
                            .filter(|s| !s.is_empty()),
                        self.listener
                            .unordered_sources
                            .split(',')
                            .filter(|s| !s.is_empty()),
                        task_limiter,
                    )
                    .await
            }
            (_, Some(amqp), _) if self.communication_method == CommunicationMethod::Amqp => {
                if self.listener.is_empty() {
                    bail!("Missing list of listener queues")
                }

                amqp.parallel_consumers(
                    self.listener
                        .ordered_sources
                        .split(',')
                        .filter(|s| !s.is_empty()),
                    self.listener
                        .unordered_sources
                        .split(',')
                        .filter(|s| !s.is_empty()),
                    task_limiter,
                )
                .await
            }
            (_, _, Some(grpc)) if self.communication_method == CommunicationMethod::GRpc => {
                Ok(vec![grpc.parallel_consumer().await?])
            }
            _ => anyhow::bail!("Unsupported consumer specification"),
        }
    }

    pub async fn publisher(&self) -> anyhow::Result<CommonPublisher> {
        publisher(
            self.kafka.as_ref().map(|kafka| kafka.brokers.as_str()),
            self.amqp.as_ref().map(|amqp| amqp.exchange_url.as_str()),
            self.grpc.as_ref().map(|_| ()),
        )
        .await
    }
}

impl KafkaSettings {
    pub async fn parallel_consumers<'a>(
        &self,
        ordered_sources: impl Iterator<Item = &'a str>,
        unordered_sources: impl Iterator<Item = &'a str>,
        task_limiter: TaskLimiter,
    ) -> anyhow::Result<Vec<ParallelCommonConsumer>> {
        let mut result = Vec::new();

        for topic in ordered_sources.chain(unordered_sources) {
            result.push(
                ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Kafka {
                    brokers: &self.brokers,
                    group_id: &self.group_id,
                    topic,
                    task_limiter: task_limiter.clone(),
                })
                .await?,
            )
        }

        Ok(result)
    }
}

impl AmqpSettings {
    pub async fn parallel_consumers<'a>(
        &self,
        ordered_sources: impl Iterator<Item = &'a str>,
        unordered_sources: impl Iterator<Item = &'a str>,
        task_limiter: TaskLimiter,
    ) -> anyhow::Result<Vec<ParallelCommonConsumer>> {
        let mut result = Vec::new();

        for queue in ordered_sources.chain(unordered_sources) {
            result.push(
                ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Amqp {
                    connection_string: &self.exchange_url,
                    consumer_tag: &self.tag,
                    queue_name: queue,
                    task_limiter: task_limiter.clone(),
                    options: self.consume_options,
                })
                .await?,
            )
        }

        Ok(result)
    }
}

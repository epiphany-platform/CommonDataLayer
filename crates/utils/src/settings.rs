use crate::communication::consumer::{CommonConsumer, CommonConsumerConfig};
use crate::communication::parallel_consumer::{
    ParallelCommonConsumer, ParallelCommonConsumerConfig,
};
use crate::communication::publisher::CommonPublisher;
use crate::notification::full_notification_sender::FullNotificationSenderBase;
use crate::notification::NotificationPublisher;
use crate::task_limiter::TaskLimiter;
use config::{Config, ConfigError, Environment, File};
use derive_more::Display;
use lapin::options::BasicConsumeOptions;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddrV4;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Deserialize, Display, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMethod {
    Kafka,
    Amqp,
    #[serde(rename = "grpc")]
    GRpc,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PostgresSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub dbname: String,
    pub schema: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VictoriaMetricsSettings {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConsumerKafkaSettings {
    pub brokers: String,
    pub group_id: String,
    pub ingest_topic: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PublisherKafkaSettings {
    pub brokers: String,
    pub egest_topic: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AmqpSettings {
    pub exchange_url: String,
    pub tag: String,
    pub ingest_queue: String,
    pub consume_options: Option<BasicConsumeOptions>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GRpcSettings {
    pub address: SocketAddrV4,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NotificationSettings {
    /// Kafka topic, AMQP queue or GRPC url
    pub destination: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MonitoringSettings {
    pub metrics_port: u16,
    pub stats_port: u16,
    pub otel_service_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LogSettings {
    pub rust_log: String,
}

pub fn load_settings<'de, T: Deserialize<'de> + Debug>() -> Result<T, ConfigError> {
    let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let exe = env::current_exe()
        .map(|f| f.file_name().map(ToOwned::to_owned))
        .unwrap()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let home = env::var("HOME").unwrap();
    let mut s = Config::new();

    s.merge(File::with_name("/etc/cdl/default.toml").required(false))?;
    s.merge(File::with_name(&format!("/etc/cdl/{}.toml", exe)).required(false))?;

    s.merge(File::with_name(&format!("{}/.cdl/{}/default.toml", home, env)).required(false))?;
    s.merge(File::with_name(&format!("{}/.cdl/{}/{}.toml", home, env, exe)).required(false))?;

    s.merge(File::with_name(&format!(".cdl/{}/default.toml", env)).required(false))?;
    s.merge(File::with_name(&format!(".cdl/{}/{}.toml", env, exe)).required(false))?;

    if let Ok(custom_dir) = env::var("CDL_CONFIG") {
        s.merge(File::with_name(&format!("{}/{}/default.toml", custom_dir, env)).required(false))?;
        s.merge(File::with_name(&format!("{}/{}/{}.toml", custom_dir, env, exe)).required(false))?;
    }

    s.merge(Environment::with_prefix(
        &exe.replace("-", "_").to_uppercase(),
    ))?;

    let settings = s.try_into()?;

    tracing::debug!(?settings, "command-line arguments");

    Ok(settings)
}
pub async fn publisher<'a>(
    kafka: Option<&'a str>,
    amqp: Option<&'a str>,
    grpc: Option<()>,
) -> anyhow::Result<CommonPublisher> {
    Ok(match (kafka, amqp, grpc) {
        (Some(brokers), _, _) => CommonPublisher::new_kafka(brokers).await?,
        (_, Some(exchange), _) => CommonPublisher::new_amqp(exchange).await?,
        (_, _, Some(_)) => CommonPublisher::new_grpc().await?,
        _ => todo!(),
    })
}

impl ConsumerKafkaSettings {
    pub async fn consumer(&self) -> anyhow::Result<CommonConsumer> {
        Ok(CommonConsumer::new(CommonConsumerConfig::Kafka {
            brokers: &self.brokers,
            group_id: &self.group_id,
            topic: self.ingest_topic.as_str(),
        })
        .await?)
    }

    pub async fn parallel_consumer(
        &self,
        task_limiter: TaskLimiter,
    ) -> anyhow::Result<ParallelCommonConsumer> {
        Ok(
            ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Kafka {
                brokers: &self.brokers,
                group_id: &self.group_id,
                topic: &self.ingest_topic,
                task_limiter,
            })
            .await?,
        )
    }
}

impl AmqpSettings {
    pub async fn consumer(&self) -> anyhow::Result<CommonConsumer> {
        Ok(CommonConsumer::new(CommonConsumerConfig::Amqp {
            connection_string: &self.exchange_url,
            consumer_tag: &self.tag,
            queue_name: &self.ingest_queue,
            options: self.consume_options,
        })
        .await?)
    }

    pub async fn parallel_consumer(
        &self,
        task_limiter: TaskLimiter,
    ) -> anyhow::Result<ParallelCommonConsumer> {
        Ok(
            ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Amqp {
                connection_string: &self.exchange_url,
                consumer_tag: &self.tag,
                queue_name: &self.ingest_queue,
                options: self.consume_options,
                task_limiter,
            })
            .await?,
        )
    }

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

impl GRpcSettings {
    pub async fn parallel_consumer(&self) -> anyhow::Result<ParallelCommonConsumer> {
        Ok(
            ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Grpc { addr: self.address })
                .await?,
        )
    }
}

impl NotificationSettings {
    pub async fn publisher<T: Serialize + Send + Sync + 'static>(
        &self,
        publisher: CommonPublisher, // FIXME
        context: String,
        application: &'static str,
    ) -> NotificationPublisher<T> {
        if self.enabled {
            NotificationPublisher::Full(
                FullNotificationSenderBase::new(
                    publisher,
                    self.destination.clone(),
                    context,
                    application,
                )
                .await,
            )
        } else {
            NotificationPublisher::Disabled
        }
    }
}

// pub fn create_notification_service(communication_method: CommunicationMethod, notification_settings: NotificationSettings) {
//     if notification_settings.enabled {
//
//     } else {
//
//     }
//     match report_config.destination {
//         Some(destination) => ReportSender::Full(
//             FullReportSenderBase::new(
//                 &communication_config,
//                 destination,
//                 output.name().to_string(),
//             )
//                 .await
//                 .map_err(Error::FailedToInitializeReporting)?,
//         ),
//         None => ReportSender::Disabled,
//     };
// }

use anyhow::bail;
use communication_utils::consumer::{CommonConsumer, CommonConsumerConfig};
use communication_utils::parallel_consumer::{
    ParallelCommonConsumer, ParallelCommonConsumerConfig,
};
use communication_utils::publisher::CommonPublisher;
use config::{Config, Environment, File};
use derive_more::Display;
use lapin::options::BasicConsumeOptions;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Debug;
use std::net::SocketAddrV4;
use task_utils::task_limiter::TaskLimiter;
use url::Url;

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
    pub url: Url,
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
pub struct MonitoringSettings {
    #[serde(default)]
    pub metrics_port: u16,
    #[serde(default = "default_status_port")]
    pub status_port: u16,
    #[serde(default = "default_otel_service_name")]
    pub otel_service_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LogSettings {
    pub rust_log: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RepositoryStaticRouting {
    pub insert_destination: String,
    pub query_address: String,
    pub repository_type: RepositoryType,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryType {
    DocumentStorage,
    Timeseries,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            rust_log: "info".to_string(),
        }
    }
}

fn default_otel_service_name() -> String {
    env::current_exe()
        .expect("Current executable name")
        .to_string_lossy()
        .to_string()
}

fn default_status_port() -> u16 {
    3000
}

pub fn load_settings<'de, T: Deserialize<'de> + Debug>() -> anyhow::Result<T> {
    let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let exe = if let Some(exe) =
        env::current_exe().map(|f| f.file_name().map(|s| s.to_string_lossy().to_string()))?
    {
        exe
    } else {
        bail!("Missing executable file name")
    };
    let mut s = Config::new();

    s.merge(File::with_name("/etc/cdl/default.toml").required(false))?;
    s.merge(File::with_name(&format!("/etc/cdl/{}.toml", exe)).required(false))?;

    s.merge(File::with_name(&format!("/etc/cdl/{}/default.toml", env)).required(false))?;
    s.merge(File::with_name(&format!("/etc/cdl/{}/{}.toml", env, exe)).required(false))?;

    if let Some(home) = dirs::home_dir() {
        s.merge(
            File::with_name(&format!("{}/.cdl/default.toml", home.to_string_lossy(),))
                .required(false),
        )?;
        s.merge(
            File::with_name(&format!("{}/.cdl/{}.toml", home.to_string_lossy(), env,))
                .required(false),
        )?;
        s.merge(
            File::with_name(&format!(
                "{}/.cdl/{}/default.toml",
                home.to_string_lossy(),
                env
            ))
            .required(false),
        )?;
        s.merge(
            File::with_name(&format!(
                "{}/.cdl/{}/{}.toml",
                home.to_string_lossy(),
                env,
                exe
            ))
            .required(false),
        )?;
    }

    s.merge(File::with_name(".cdl/default.toml").required(false))?;
    s.merge(File::with_name(&format!(".cdl/{}.toml", exe)).required(false))?;
    s.merge(File::with_name(&format!(".cdl/{}/default.toml", env)).required(false))?;
    s.merge(File::with_name(&format!(".cdl/{}/{}.toml", env, exe)).required(false))?;

    if let Ok(custom_dir) = env::var("CDL_CONFIG") {
        s.merge(File::with_name(&format!("{}/default.toml", custom_dir)).required(false))?;
        s.merge(File::with_name(&format!("{}/{}.toml", custom_dir, exe)).required(false))?;
        s.merge(File::with_name(&format!("{}/{}/default.toml", custom_dir, env)).required(false))?;
        s.merge(File::with_name(&format!("{}/{}/{}.toml", custom_dir, env, exe)).required(false))?;
    }

    s.merge(Environment::with_prefix(&exe.replace("-", "_")).separator("__"))?;

    let settings = s.try_into()?;

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
        _ => anyhow::bail!("Unsupported publisher specification"),
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
}

impl GRpcSettings {
    pub async fn parallel_consumer(&self) -> anyhow::Result<ParallelCommonConsumer> {
        Ok(
            ParallelCommonConsumer::new(ParallelCommonConsumerConfig::Grpc { addr: self.address })
                .await?,
        )
    }
}

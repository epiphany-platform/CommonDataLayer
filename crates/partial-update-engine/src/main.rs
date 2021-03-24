use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;
use tracing::{debug, error, trace};
use utils::{
    communication::{
        consumer::{CommonConsumer, CommonConsumerConfig, ConsumerHandler},
        message::CommunicationMessage,
        publisher::CommonPublisher,
    },
    metrics::{self},
};

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub kafka_brokers: String,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub kafka_group_id: String,
    /// Kafka topic
    #[structopt(long, env)]
    pub notification_topic: String,
    /// Address of schema registry gRPC API
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    /// How many entries the cache can hold
    #[structopt(long, env)]
    pub cache_capacity: usize,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve(config.metrics_port);

    let notification_consumer = notification_consumer(&config).await?;
    let update_view_producer = Arc::new(update_view_producer(&config).await?);

    let schema_registry_addr = Arc::new(config.schema_registry_addr);

    notification_consumer.run(Handler {}).await?;

    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

struct Handler {}

#[async_trait]
impl ConsumerHandler for Handler {
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        todo!()
    }
}
async fn update_view_producer(config: &Config) -> anyhow::Result<CommonPublisher> {
    Ok(CommonPublisher::new_kafka(&config.kafka_brokers).await?)
}

async fn notification_consumer(config: &Config) -> anyhow::Result<CommonConsumer> {
    debug!("Initializing Kafka consumer");
    let config = {
        CommonConsumerConfig::Kafka {
            brokers: &config.kafka_brokers,
            group_id: &config.kafka_group_id,
            topic: &config.notification_topic,
        }
    };
    Ok(CommonConsumer::new(config).await?)
}

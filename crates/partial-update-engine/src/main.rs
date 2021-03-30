use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::{Context, Result};
use rdkafka::{ClientConfig, Message, Offset, TopicPartitionList, consumer::{CommitMode, DefaultConsumerContext, StreamConsumer}, error::KafkaError, message::BorrowedMessage};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::{stream::StreamExt, time::delay_for};
use tracing::debug;
use utils::metrics::{self};
use uuid::Uuid;

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub brokers: String,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub group_id: String,
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
    /// Duration of sleep phase in seconds
    #[structopt(long, env)]
    pub sleep_phase_length: u64,
}

#[derive(Deserialize, PartialEq, Eq,Hash)]
struct PartialNotification{
    pub object_id: Uuid,
    pub schema_id: Uuid
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve(config.metrics_port);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", &config.group_id)
        .set("bootstrap.servers", &config.brokers)
        .set("enable.partition.eof", "true")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .context("Consumer creation failed")?;
    let topics = [config.notification_topic.as_str()];

    rdkafka::consumer::Consumer::subscribe(&consumer, &topics)
        .context("Can't subscribe to specified topics")?;

    let mut message_stream = consumer.start();

    let mut changes: HashSet<PartialNotification> = HashSet::new();
    let mut offsets: HashMap<i32, i64> = HashMap::new();
    loop {
        match message_stream.try_next().await {
            Ok(opt_message) => match opt_message {
                Some(message) => {
                    new_notification(&mut changes, message)?;
                }
                None => {
                    process_changes(&mut changes).await?;
                    acknowledge_messages(&mut offsets, &consumer, &config.notification_topic)
                        .await?;
                    break;
                }
            },
            Err(err) => match err {
                KafkaError::PartitionEOF(_) => {
                    // TODO: What if eof on one partition, not on others?
                    process_changes(&mut changes).await?;
                    acknowledge_messages(&mut offsets, &consumer, &config.notification_topic)
                        .await?;
                    debug!("Entering sleep phase");
                    delay_for(Duration::from_secs(config.sleep_phase_length)).await;
                    debug!("Exiting sleep phase");
                }
                err => {
                    panic!("Unexpected kafka error {:?}", err)
                }
            },
        }
    }

    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}


fn new_notification(changes: &mut HashSet<PartialNotification>, message: BorrowedMessage) -> Result<()> {
    let notification: PartialNotification = serde_json::from_str(
        message.payload_view::<str>().ok_or_else(|| anyhow::anyhow!("Message has no payload"))??,
    )?;
    changes.insert(notification);
    Ok(())
}

async fn process_changes(changes: &mut HashSet<PartialNotification>) -> Result<()> {
    // TODO: Send changes to object_builder
    todo!();
    changes.clear();
    Ok(())
}

async fn acknowledge_messages(
    offsets: &mut HashMap<i32, i64>,
    consumer: &StreamConsumer,
    notification_topic: &str,
) -> Result<()> {
    let mut partition_offsets = TopicPartitionList::new();
    for offset in offsets.iter() {
        partition_offsets.add_partition_offset(
            notification_topic,
            *offset.0,
            Offset::Offset(*offset.1),
        );
    }
    rdkafka::consumer::Consumer::commit(consumer, &partition_offsets, CommitMode::Sync)?;
    offsets.clear();
    Ok(())
}

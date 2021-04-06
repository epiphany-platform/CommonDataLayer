use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::{Context, Result};
use rdkafka::{
    consumer::{CommitMode, DefaultConsumerContext, StreamConsumer},
    message::BorrowedMessage,
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message, Offset, TopicPartitionList,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::debug;
use utils::metrics::{self};
use uuid::Uuid;

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub kafka_brokers: String,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub kafka_group_id: String,
    /// Kafka topic for notifications
    #[structopt(long, env)]
    pub notification_topic: String,
    /// Kafka topic for object builder input
    #[structopt(long, env)]
    pub object_builder_topic: String,
    /// Address of schema registry gRPC API
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Duration of sleep phase in seconds
    #[structopt(long, env)]
    pub sleep_phase_length: u64,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
struct PartialNotification {
    pub object_id: Uuid,
    pub schema_id: Uuid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve(config.metrics_port);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", &config.kafka_group_id)
        .set("bootstrap.servers", &config.kafka_brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("enable.auto.offset.store", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .context("Consumer creation failed")?;
    let topics = [config.notification_topic.as_str()];

    rdkafka::consumer::Consumer::subscribe(&consumer, &topics)
        .context("Can't subscribe to specified topics")?;

    let producer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka_brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .set("compression.type", "none")
        .set("max.in.flight.requests.per.connection", "5")
        .create()?;

    let mut message_stream = Box::pin(consumer.stream().timeout(Duration::from_secs(2)));// TODO: configure?
    let mut changes: HashSet<PartialNotification> = HashSet::new();
    let mut offsets: HashMap<i32, i64> = HashMap::new();
    loop {
        // TODO: configure max items per batch(?) - otherwise we won't start view recalculation if messages are sent more often then timeout
        match message_stream.try_next().await {
            Ok(opt_message) => match opt_message {
                Some(message) => {
                    new_notification(&mut changes, message?)?;
                }
                None => {
                    process_changes(&producer, &config, &mut changes).await?;
                    acknowledge_messages(&mut offsets, &consumer, &config.notification_topic)
                        .await?;
                    break;
                }
            },
            Err(_) => {
                if !changes.is_empty(){
                    process_changes(&producer, &config, &mut changes).await?;
                    acknowledge_messages(&mut offsets, &consumer, &config.notification_topic).await?;
                }
                debug!("Entering sleep phase");
                sleep(Duration::from_secs(config.sleep_phase_length)).await;
                debug!("Exiting sleep phase");
            }
        }
    }

    sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

fn new_notification(
    changes: &mut HashSet<PartialNotification>,
    message: BorrowedMessage,
) -> Result<()> {
    let notification: PartialNotification = serde_json::from_str(
        message
            .payload_view::<str>()
            .ok_or_else(|| anyhow::anyhow!("Message has no payload"))??,
    )?;
    changes.insert(notification);
    Ok(())
}

async fn process_changes(
    producer: &FutureProducer,
    config: &Config,
    changes: &mut HashSet<PartialNotification>,
) -> Result<()> {
    let schemas: HashSet<_> = changes.iter().map(|x| x.schema_id).collect();
    let mut client = rpc::schema_registry::connect(config.schema_registry_addr.to_owned()).await?;
    let mut views: HashSet<Uuid> = HashSet::new();
    for schema in schemas {
        let response = client
            .get_all_views_of_schema(rpc::schema_registry::Id {
                id: schema.to_string(),
            })
            .await?;
        let schema_in_views = response
            .into_inner()
            .views
            .iter()
            .map(|view| Ok(view.0.parse()?))
            .collect::<Result<Vec<Uuid>>>()?;
        for view in schema_in_views {
            views.insert(view);
        }
    }

    for view in views {
        let payload = format!("{{{}}}", view); // TODO: Use same type as object builder
        producer
            .send(
                FutureRecord::to(config.object_builder_topic.as_str())
                    .payload(payload.as_str())
                    .key(&view.to_string()),
                Duration::from_secs(5),
            )
            .await
            .map_err(|err| anyhow::anyhow!("Error sending message to Kafka {:?}", err))?;
    }
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
        )?;
    }
    rdkafka::consumer::Consumer::commit(consumer, &partition_offsets, CommitMode::Sync)?;
    offsets.clear();
    Ok(())
}

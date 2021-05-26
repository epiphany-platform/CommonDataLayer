use anyhow::{Context, Result};
use rdkafka::consumer::Consumer;
use rdkafka::{
    consumer::{CommitMode, DefaultConsumerContext, StreamConsumer},
    message::{BorrowedMessage, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message, Offset, TopicPartitionList,
};
use rpc::schema_registry::{FullView, Id};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{trace, Instrument};
use utils::metrics::{self};
use utils::settings::*;
use utils::types::materialization::Request;
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
struct Settings {
    communication_method: CommunicationMethod,
    sleep_phase_length: u64,

    kafka: PublisherKafkaSettings,
    notification_consumer: NotificationConsumerSettings,
    services: ServicesSettings,

    monitoring: MonitoringSettings,

    log: LogSettings,
}

#[derive(Deserialize, Debug, Serialize)]
struct NotificationConsumerSettings {
    pub brokers: String,
    pub group_id: String,
    pub source: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct ServicesSettings {
    pub schema_registry_url: String,
    pub edge_registry_url: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
enum PartialNotification {
    CommandServiceNotification(CommandServiceNotification),
    EdgeRegistryNotification(EdgeRegistryNotification),
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct CommandServiceNotification {
    pub object_id: Uuid,
    pub schema_id: Uuid,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
struct EdgeRegistryNotification {
    pub relation_id: Uuid,
    pub parent_object_id: Uuid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    ::utils::tracing::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    metrics::serve(&settings.monitoring);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", &settings.notification_consumer.group_id)
        .set("bootstrap.servers", &settings.notification_consumer.brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .create()
        .context("Consumer creation failed")?;
    let topics = [settings.notification_consumer.source.as_str()];

    consumer
        .subscribe(&topics)
        .context("Can't subscribe to specified topics")?;

    let producer = ClientConfig::new()
        .set("bootstrap.servers", &settings.kafka.brokers)
        .set("message.timeout.ms", "5000")
        .set("acks", "all")
        .set("compression.type", "none")
        .set("max.in.flight.requests.per.connection", "5")
        .create()?;

    let mut message_stream = Box::pin(consumer.stream().timeout(Duration::from_secs(2))); // TODO: configure?
    let mut changes: HashSet<PartialNotification> = HashSet::new();
    let mut offsets: HashMap<i32, i64> = HashMap::new();
    loop {
        // TODO: configure max items per batch(?) - otherwise we won't start view recalculation if messages are sent more often then timeout
        match message_stream.try_next().await {
            Ok(opt_message) => match opt_message {
                Some(message) => {
                    let (partition, offset) = new_notification(&mut changes, message?)?;
                    offsets.insert(partition, offset);
                }
                None => {
                    process_changes(&producer, &settings, &mut changes).await?;
                    acknowledge_messages(
                        &mut offsets,
                        &consumer,
                        &settings.notification_consumer.source,
                    )
                    .await?;
                    break;
                }
            },
            Err(_) => {
                trace!("Timeout");
                if !changes.is_empty() {
                    process_changes(&producer, &settings, &mut changes).await?;
                    acknowledge_messages(
                        &mut offsets,
                        &consumer,
                        &settings.notification_consumer.source,
                    )
                    .await?;
                }
                let sleep_phase = tracing::info_span!("Sleep phase");
                sleep(Duration::from_secs(settings.sleep_phase_length))
                    .instrument(sleep_phase)
                    .await;
            }
        }
    }

    sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

#[tracing::instrument(skip(message))]
fn new_notification(
    changes: &mut HashSet<PartialNotification>,
    message: BorrowedMessage,
) -> Result<(i32, i64)> {
    utils::tracing::kafka::set_parent_span(&message);
    let payload = message
        .payload_view::<str>()
        .ok_or_else(|| anyhow::anyhow!("Message has no payload"))??;

    let notification: PartialNotification = serde_json::from_str(payload)?;
    trace!("new notification {:#?}", notification);
    changes.insert(notification);
    let partition = message.partition();
    let offset = message.offset();
    Ok((partition, offset))
}

#[tracing::instrument(skip(producer, settings))]
async fn process_changes(
    producer: &FutureProducer,
    settings: &Settings,
    changes: &mut HashSet<PartialNotification>,
) -> Result<()> {
    trace!("processing changes {:#?}", changes);
    let mut sr_client =
        rpc::schema_registry::connect(settings.services.schema_registry_url.to_owned()).await?;

    let mut schema_cache: HashMap<Uuid, Vec<FullView>> = HashMap::default();
    let mut relation_cache: HashMap<Uuid, Vec<FullView>> = HashMap::default();
    let mut requests: HashMap<Uuid, Request> = HashMap::default();

    for change in changes.drain() {
        match change {
            PartialNotification::CommandServiceNotification(notification) => {
                // New object or new object version was added
                let entry = schema_cache.entry(notification.schema_id);
                let views = match entry {
                    Entry::Occupied(ref entry) => entry.get(),
                    Entry::Vacant(entry) => {
                        let response = sr_client
                            .get_all_views_of_schema(Id {
                                id: notification.schema_id.to_string(),
                            })
                            .await?
                            .into_inner();

                        entry.insert(response.views)
                    }
                };

                for view in views {
                    let view_id = view.id.parse()?;
                    requests
                        .entry(view_id)
                        .or_insert_with(|| Request::new(view_id))
                        .schemas
                        .entry(notification.schema_id)
                        .or_default()
                        .object_ids
                        .insert(notification.object_id);
                }
            }
            PartialNotification::EdgeRegistryNotification(notification) => {
                let entry = relation_cache.entry(notification.relation_id);
                let views = match entry {
                    Entry::Occupied(ref entry) => entry.get(),
                    Entry::Vacant(entry) => {
                        let response = sr_client
                            .get_all_views_by_relation(Id {
                                id: notification.relation_id.to_string(),
                            })
                            .await?
                            .into_inner();

                        entry.insert(response.views)
                    }
                };

                for view in views {
                    let view_id = view.id.parse()?;
                    requests
                        .entry(view_id)
                        .or_insert_with(|| Request::new(view_id))
                        .schemas
                        .entry(view.base_schema_id.parse()?)
                        .or_default()
                        .object_ids
                        .insert(notification.parent_object_id);
                }
            }
        }
    }

    trace!(?requests, "Requests");

    for request in requests.values() {
        let payload = serde_json::to_string(&request)?;
        producer
            .send(
                FutureRecord::to(settings.kafka.egest_topic.as_str())
                    .payload(payload.as_str())
                    .key(&request.view_id.to_string())
                    .headers(utils::tracing::kafka::inject_span(OwnedHeaders::new())),
                Duration::from_secs(5),
            )
            .await
            .map_err(|err| anyhow::anyhow!("Error sending message to Kafka {:?}", err))?;
    }

    Ok(())
}

#[tracing::instrument(skip(consumer))]
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
            Offset::Offset(*offset.1 + 1),
        )?;
    }
    rdkafka::consumer::Consumer::commit(consumer, &partition_offsets, CommitMode::Sync)?;
    offsets.clear();
    Ok(())
}

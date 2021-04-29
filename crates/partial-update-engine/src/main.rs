use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::{Context, Result};
use rdkafka::{
    consumer::{CommitMode, DefaultConsumerContext, StreamConsumer},
    message::{BorrowedMessage, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message, Offset, TopicPartitionList,
};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{debug, trace, Instrument};
use utils::settings::*;
use utils::{
    metrics::{self},
    types::materialization,
};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
struct Settings {
    communication_method: CommunicationMethod,
    sleep_phase_length: u64,

    kafka: PublisherKafkaSettings,
    notification_consumer: NotificationConsumerSettings,
    services: ServicesSettings,

    monitoring: MonitoringSettings,
}

#[derive(Deserialize, Debug, Serialize)]
struct NotificationConsumerSettings {
    #[serde(flatten)]
    pub kafka: ConsumerKafkaSettings,
    pub source: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct ServicesSettings {
    pub schema_registry_url: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
struct PartialNotification {
    pub object_id: Uuid,
    pub schema_id: Uuid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let settings: Settings = load_settings()?;

    debug!("Environment {:?}", settings);

    metrics::serve(&settings.monitoring);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", &settings.notification_consumer.kafka.group_id)
        .set(
            "bootstrap.servers",
            &settings.notification_consumer.kafka.brokers,
        )
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .create()
        .context("Consumer creation failed")?;
    let topics = [settings.notification_consumer.source.as_str()];

    rdkafka::consumer::Consumer::subscribe(&consumer, &topics)
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
    let notification: PartialNotification = serde_json::from_str(
        message
            .payload_view::<str>()
            .ok_or_else(|| anyhow::anyhow!("Message has no payload"))??,
    )?;
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
    let mut client =
        rpc::schema_registry::connect(settings.services.schema_registry_url.to_owned()).await?;
    let mut schema_cache: HashMap<Uuid, Vec<Uuid>> // (Schema_ID, Vec<View_ID>)
        = Default::default();

    let mut requests: HashMap<Uuid, materialization::Request> = Default::default();

    for PartialNotification {
        object_id,
        schema_id,
    } in std::mem::take(changes).into_iter()
    {
        let entry = schema_cache.entry(schema_id);
        let view_ids = match entry {
            std::collections::hash_map::Entry::Occupied(ref entry) => entry.get(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let response = client
                    .get_all_views_of_schema(rpc::schema_registry::Id {
                        id: schema_id.to_string(),
                    })
                    .await?;

                trace!(?response, "Response");

                let view_ids = response
                    .into_inner()
                    .views
                    .iter()
                    .map(|view| Ok(view.id.parse()?))
                    .collect::<Result<Vec<Uuid>>>()?;

                entry.insert(view_ids)
            }
        };

        for view_id in view_ids {
            requests
                .entry(*view_id)
                .or_insert_with(|| materialization::Request::new(*view_id))
                .schemas
                .entry(schema_id)
                .or_default()
                .object_ids
                .insert(object_id);
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

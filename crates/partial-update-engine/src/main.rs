use anyhow::{Context, Result};
use itertools::Itertools;
use rdkafka::consumer::Consumer;
use rdkafka::{
    consumer::{CommitMode, DefaultConsumerContext, StreamConsumer},
    message::{BorrowedMessage, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    ClientConfig, Message, Offset, TopicPartitionList,
};
use rpc::edge_registry::{RelationId, TreeQuery, TreeResponse};
use rpc::schema_registry::{FullView, Relation};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{trace, Instrument};
use utils::settings::*;
use utils::types::materialization::{Request, Schema};
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
    let mut er_client =
        rpc::edge_registry::connect(settings.services.edge_registry_url.to_owned()).await?;
    let mut schema_cache: HashMap<Uuid, Vec<Uuid>> // (Schema_ID, Vec<View_ID>)
        = Default::default();
    let mut view_cache: HashMap<Uuid, FullView> = Default::default();
    let mut temp: HashMap<Uuid, HashMap<Uuid, HashSet<Uuid>>> = Default::default(); // (ViewID -> SchemaID -> [ObjectId])
    let mut requests: HashMap<Uuid, materialization::Request> = Default::default();

    let mut new_changes = Vec::with_capacity(changes.len());

    // Convert notifications to (schema, object) pairs
    for change in changes.drain() {
        match change {
            PartialNotification::CommandServiceNotification(notification) => {
                new_changes.push((notification.schema_id, notification.object_id));
            }
            PartialNotification::EdgeRegistryNotification(notification) => {
                let relation = er_client
                    .get_schema_by_relation(RelationId {
                        relation_id: notification.relation_id.to_string(),
                    })
                    .await?
                    .into_inner();
                new_changes.push((
                    relation.parent_schema_id.parse()?,
                    notification.parent_object_id,
                ));
            }
        }
    }

    // Group objects per view per schema
    for (key, group) in new_changes.into_iter().group_by(|(k, _)| *k).into_iter() {
        let entry = schema_cache.entry(key);
        let view_ids = match entry {
            Entry::Occupied(ref occupied) => occupied.get(),
            Entry::Vacant(vacant) => {
                let schema_views = sr_client
                    .get_all_views_of_schema(rpc::schema_registry::Id {
                        id: key.to_string(),
                    })
                    .await?
                    .into_inner();

                let view_ids = vacant.insert(
                    schema_views
                        .views
                        .iter()
                        .map(|item| item.id.parse())
                        .collect::<Result<_, _>>()?,
                );
                for view in schema_views.views {
                    view_cache.entry(view.id.parse()?).or_insert(view);
                }
                view_ids
            }
        };

        let object_ids: Vec<Uuid> = group.into_iter().map(|item| item.1).collect();

        for view_id in view_ids {
            temp.entry(*view_id)
                .or_default()
                .entry(key)
                .or_default()
                .extend(object_ids.iter())
        }
    }

    // Query ER for relation trees of each view
    for (view_id, schemas) in temp {
        let full_view = view_cache.get(&view_id).unwrap();
        for (schema_id, object_ids) in schemas {
            for query in view_to_tree_query(
                full_view,
                object_ids.into_iter().map(|i| i.to_string()).collect(),
            ) {
                let schema = requests
                    .entry(view_id)
                    .or_insert(Request {
                        view_id,
                        ..Default::default()
                    })
                    .schemas
                    .entry(schema_id)
                    .or_default();
                process_tree_response(er_client.resolve_tree(query).await?.into_inner(), schema)?;
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

fn view_to_tree_query(
    view: &'_ FullView,
    object_ids: Vec<String>,
) -> impl Iterator<Item = TreeQuery> + '_ {
    view.relations.iter().map(move |relation| TreeQuery {
        relation_id: relation.global_id.to_string(),
        relations: relation
            .relations
            .iter()
            .map(relation_to_tree_query)
            .collect(),
        filter_ids: object_ids.clone(),
    })
}

fn relation_to_tree_query(relation: &Relation) -> TreeQuery {
    TreeQuery {
        relation_id: relation.global_id.to_owned(),
        relations: relation
            .relations
            .iter()
            .map(|relation| relation_to_tree_query(relation))
            .collect(),
        filter_ids: vec![],
    }
}

fn process_tree_response(tree_response: TreeResponse, schema: &mut Schema) -> Result<()> {
    for object in tree_response.objects {
        schema.object_ids.insert(object.object_id.parse().unwrap());
        for child in object.children {
            schema.object_ids.insert(child.parse()?);
        }
        for subtree in object.subtrees {
            process_tree_response(subtree, schema)?;
        }
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

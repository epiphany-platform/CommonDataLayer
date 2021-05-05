use anyhow::Context;
use async_trait::async_trait;
use lru_cache::LruCache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::{error, trace};

use rpc::schema_registry::Id;
use utils::settings::{
    load_settings, AmqpSettings, ConsumerKafkaSettings, GRpcSettings, LogSettings,
    MonitoringSettings,
};
use utils::{
    abort_on_poison,
    communication::{
        get_order_group_id,
        message::CommunicationMessage,
        parallel_consumer::{ParallelCommonConsumer, ParallelConsumerHandler},
        publisher::CommonPublisher,
    },
    current_timestamp,
    message_types::BorrowedInsertMessage,
    message_types::DataRouterInsertMessage,
    metrics::{self, counter},
    parallel_task_queue::ParallelTaskQueue,
    task_limiter::TaskLimiter,
};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CommunicationMethod {
    Kafka,
    Amqp,
    #[serde(rename = "grpc")]
    GRpc,
}

#[derive(Deserialize, Debug, Serialize)]
struct Settings {
    communication_method: CommunicationMethod,
    cache_capacity: usize,
    async_task_limit: usize,

    kafka: Option<ConsumerKafkaSettings>,
    amqp: Option<AmqpSettings>,
    grpc: Option<GRpcSettings>,

    monitoring: MonitoringSettings,

    services: ServicesSettings,

    log: LogSettings,
}

#[derive(Deserialize, Debug, Serialize)]
struct ServicesSettings {
    schema_registry_url: String,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    ::utils::tracing::init(settings.log.rust_log.as_str())?;

    tracing::debug!(?settings, "command-line arguments");

    metrics::serve(&settings.monitoring);

    let consumer = settings.consumer().await?;
    let producer = Arc::new(settings.producer().await?);

    let cache = Arc::new(Mutex::new(LruCache::new(settings.cache_capacity)));
    let schema_registry_addr = Arc::new(settings.services.schema_registry_url);

    let task_queue = Arc::new(ParallelTaskQueue::default());

    consumer
        .par_run(Handler {
            cache,
            producer,
            schema_registry_addr,
            task_queue,
        })
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

struct Handler {
    cache: Arc<Mutex<LruCache<Uuid, String>>>,
    producer: Arc<CommonPublisher>,
    schema_registry_addr: Arc<String>,
    task_queue: Arc<ParallelTaskQueue>,
}

#[async_trait]
impl ParallelConsumerHandler for Handler {
    #[tracing::instrument(skip(self, message))]
    async fn handle<'a>(&'a self, message: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let order_group_id = get_order_group_id(message);
        let _guard =
            order_group_id.map(move |x| async move { self.task_queue.acquire_permit(x).await });

        trace!(
            "Received message ({:?}) `{:?}`",
            message.key(),
            message.payload()
        );

        let message_key = get_order_group_id(message).unwrap_or_default();
        counter!("cdl.data-router.input-msg", 1);
        let result = async {
            let json_something: Value = serde_json::from_str(message.payload()?)
                .context("Payload deserialization failed")?;
            if json_something.is_array() {
                trace!("Processing multimessage");

                let maybe_array: Vec<DataRouterInsertMessage> = serde_json::from_str(
                    message.payload()?,
                )
                .context("Payload deserialization failed, message is not a valid cdl message ")?;

                let mut result = Ok(());

                for entry in maybe_array.iter() {
                    let r = route(
                        &self.cache,
                        entry,
                        &message_key,
                        &self.producer,
                        &self.schema_registry_addr,
                    )
                    .await
                    .context("Tried to send message and failed");

                    counter!("cdl.data-router.input-multimsg", 1);
                    counter!("cdl.data-router.processed", 1);

                    if r.is_err() {
                        result = r;
                    }
                }

                result
            } else {
                trace!("Processing single message");

                let owned: DataRouterInsertMessage =
                    serde_json::from_str::<DataRouterInsertMessage>(message.payload()?).context(
                        "Payload deserialization failed, message is not a valid cdl message",
                    )?;
                let result = route(
                    &self.cache,
                    &owned,
                    &message_key,
                    &self.producer,
                    &self.schema_registry_addr,
                )
                .await
                .context("Tried to send message and failed");
                counter!("cdl.data-router.input-singlemsg", 1);
                counter!("cdl.data-router.processed", 1);

                result
            }
        }
        .await;

        counter!("cdl.data-router.input-request", 1);

        if let Err(error) = result {
            counter!("cdl.data-router.error", 1);

            return Err(error);
        } else {
            counter!("cdl.data-router.success", 1);
        }

        Ok(())
    }
}

#[tracing::instrument(skip(producer))]
async fn route(
    cache: &Mutex<LruCache<Uuid, String>>,
    event: &DataRouterInsertMessage<'_>,
    message_key: &str,
    producer: &CommonPublisher,
    schema_registry_addr: &str,
) -> anyhow::Result<()> {
    let payload = BorrowedInsertMessage {
        object_id: event.object_id,
        schema_id: event.schema_id,
        timestamp: current_timestamp(),
        data: event.data,
    };

    let mut conn = rpc::schema_registry::connect(schema_registry_addr.to_owned()).await?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: event.schema_id.to_string(),
        })
        .await
        .context("failed to get schema metadata")?
        .into_inner();

    send_message(
        producer,
        &schema.insert_destination,
        message_key,
        serde_json::to_vec(&payload)?,
    )
    .await;
    Ok(())
}

#[tracing::instrument(skip(cache))]
async fn get_schema_insert_destination(
    cache: &Mutex<LruCache<Uuid, String>>,
    schema_id: Uuid,
    schema_addr: &str,
) -> anyhow::Result<String> {
    let recv_channel = cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .get_mut(&schema_id)
        .cloned();
    if let Some(val) = recv_channel {
        trace!("Retrieved insert destination for {} from cache", schema_id);
        return Ok(val);
    }

    let mut client = rpc::schema_registry::connect(schema_addr.to_owned()).await?;
    let channel = client
        .get_schema_metadata(Id {
            id: schema_id.to_string(),
        })
        .await?
        .into_inner()
        .insert_destination;

    trace!(
        "Retrieved insert destination for {} from schema registry",
        schema_id
    );
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(schema_id, channel.clone());

    Ok(channel)
}

#[tracing::instrument(skip(producer))]
async fn send_message(
    producer: &CommonPublisher,
    insert_destination: &str,
    key: &str,
    payload: Vec<u8>,
) {
    let payload_len = payload.len();
    let delivery_status = producer
        .publish_message(&insert_destination, key, payload)
        .await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Insert destination: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            insert_destination, key, payload_len, delivery_status
        );
    } else {
        counter!("cdl.data-router.output-singleok", 1);
    }
}

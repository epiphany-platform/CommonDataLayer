use anyhow::Context;
use async_trait::async_trait;
use log::{debug, error, trace};
use lru_cache::LruCache;
use rpc::schema_registry::Id;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process,
    sync::{Arc, Mutex},
};
use structopt::{clap::arg_enum, StructOpt};
use utils::{
    abort_on_poison,
    communication::{
        get_order_group_id,
        message::CommunicationMessage,
        parallel_consumer::{
            ParallelCommonConsumer, ParallelCommonConsumerConfig, ParallelConsumerHandler,
        },
        publisher::CommonPublisher,
    },
    message_types::BorrowedInsertMessage,
    metrics::{self, counter},
    parallel_task_queue::ParallelTaskQueue,
    task_limiter::TaskLimiter,
};
use utils::{current_timestamp, message_types::DataRouterInsertMessage};
use uuid::Uuid;

arg_enum! {
    #[derive(Deserialize, Debug, Serialize)]
    enum CommunicationMethod {
        Kafka,
        Amqp,
        Grpc
    }
}

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    /// The method of communication with external services
    #[structopt(long, env, possible_values = &CommunicationMethod::variants(), case_insensitive = true)]
    pub communication_method: CommunicationMethod,
    /// Address of Kafka brokers
    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    /// Group ID of the consumer
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    /// Connection URL to AMQP Server
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    /// Consumer tag
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,
    /// Kafka topic or AMQP queue
    #[structopt(long, env)]
    pub input_source: Option<String>,
    /// Address of schema registry gRPC API
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    /// How many entries the cache can hold
    #[structopt(long, env)]
    pub cache_capacity: usize,
    /// Max requests handled in parallel
    #[structopt(long, env, default_value = "128")]
    pub task_limit: usize,
    /// Port to listen on for Prometheus requests
    #[structopt(default_value = metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Port to listen on when communication method is `grpc`
    #[structopt(long, env)]
    pub grpc_port: Option<u16>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve(config.metrics_port);

    let consumer = new_consumer(&config).await?;
    let producer = Arc::new(new_producer(&config).await?);

    let cache = Arc::new(Mutex::new(LruCache::new(config.cache_capacity)));

    let schema_registry_addr = Arc::new(config.schema_registry_addr);

    let task_queue = Arc::new(ParallelTaskQueue::default());

    consumer
        .par_run(Handler {
            cache,
            producer,
            schema_registry_addr,
            task_queue,
        })
        .await?;

    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

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
                        &entry,
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

async fn new_producer(config: &Config) -> anyhow::Result<CommonPublisher> {
    Ok(match config.communication_method {
        CommunicationMethod::Kafka => {
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            CommonPublisher::new_kafka(brokers).await?
        }
        CommunicationMethod::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            CommonPublisher::new_amqp(connection_string).await?
        }
        CommunicationMethod::Grpc => CommonPublisher::new_grpc("command_service").await?,
    })
}

async fn new_consumer(config: &Config) -> anyhow::Result<ParallelCommonConsumer> {
    let config = match config.communication_method {
        CommunicationMethod::Kafka => {
            let topic = config
                .input_source
                .as_ref()
                .context("kafka topic was not specified")?;
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            let group_id = config
                .kafka_group_id
                .as_ref()
                .context("kafka group was not specified")?;

            debug!("Initializing Kafka consumer");

            ParallelCommonConsumerConfig::Kafka {
                brokers: &brokers,
                group_id: &group_id,
                task_limiter: TaskLimiter::new(config.task_limit),
                topic,
            }
        }
        CommunicationMethod::Amqp => {
            let queue_name = config
                .input_source
                .as_ref()
                .context("amqp queue name was not specified")?;
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            let consumer_tag = config
                .amqp_consumer_tag
                .as_ref()
                .context("amqp consumer tag was not specified")?;

            debug!("Initializing Amqp consumer");

            ParallelCommonConsumerConfig::Amqp {
                connection_string: &connection_string,
                task_limiter: TaskLimiter::new(config.task_limit),
                consumer_tag: &consumer_tag,
                queue_name,
                options: None,
            }
        }
        CommunicationMethod::Grpc => {
            let port = config
                .grpc_port
                .clone()
                .context("grpc port was not specified")?;

            let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
            ParallelCommonConsumerConfig::Grpc { addr }
        }
    };
    Ok(ParallelCommonConsumer::new(config).await?)
}

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
    let topic_name = get_schema_topic(&cache, payload.schema_id, &schema_registry_addr).await?;

    send_message(
        producer,
        &topic_name,
        message_key,
        serde_json::to_vec(&payload)?,
    )
    .await;
    Ok(())
}

async fn get_schema_topic(
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
        trace!("Retrieved topic for {} from cache", schema_id);
        return Ok(val);
    }

    let mut client = rpc::schema_registry::connect(schema_addr.to_owned()).await?;
    let channel = client
        .get_schema_topic(Id {
            id: schema_id.to_string(),
        })
        .await?
        .into_inner()
        .topic;

    trace!("Retrieved topic for {} from schema registry", schema_id);
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(schema_id, channel.clone());

    Ok(channel)
}

async fn send_message(producer: &CommonPublisher, topic_name: &str, key: &str, payload: Vec<u8>) {
    let payload_len = payload.len();
    let delivery_status = producer.publish_message(&topic_name, key, payload).await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Topic: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            topic_name, key, payload_len, delivery_status
        );
        process::abort();
    };
    counter!("cdl.data-router.output-singleok", 1);
}

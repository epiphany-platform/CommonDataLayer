use anyhow::Context;
use log::{error, trace};
use lru_cache::LruCache;
use rpc::schema_registry::Id;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    process,
    sync::{Arc, Mutex},
};
use structopt::{clap::arg_enum, StructOpt};
use tokio::pin;
use tokio::stream::StreamExt;
use utils::{
    abort_on_poison,
    message_types::BorrowedInsertMessage,
    messaging_system::{
        consumer::CommonConsumer, message::CommunicationMessage, publisher::CommonPublisher,
    },
    metrics::{self, counter},
};
use utils::{
    message_types::DataRouterInsertMessage, messaging_system::consumer::CommonConsumerConfig,
};
use uuid::Uuid;

const SERVICE_NAME: &str = "data-router";

arg_enum! {
    #[derive(Deserialize, Debug, Serialize)]
    enum MessageQueueKind {
        Kafka,
        Amqp
    }
}

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    #[structopt(long, env, possible_values = &MessageQueueKind::variants(), case_insensitive = true)]
    pub message_queue: MessageQueueKind,
    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,
    #[structopt(long, env)]
    pub input_topic_or_queue: String,
    #[structopt(long, env)]
    pub error_topic_or_exchange: String,
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    #[structopt(long, env)]
    pub cache_capacity: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config: Config = Config::from_args();
    metrics::serve();

    let consumer = new_consumer(&config, &config.input_topic_or_queue).await?;
    let producer = Arc::new(new_producer(&config).await?);

    let cache = Arc::new(Mutex::new(LruCache::new(config.cache_capacity)));
    let consumer = consumer.leak();
    let message_stream = consumer.consume().await;
    pin!(message_stream);

    let error_topic_or_exchange = Arc::new(config.error_topic_or_exchange);
    let schema_registry_addr = Arc::new(config.schema_registry_addr);

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(message) => {
                handle_message(
                    message,
                    cache.clone(),
                    producer.clone(),
                    error_topic_or_exchange.clone(),
                    schema_registry_addr.clone(),
                )
                .await;
            }
            Err(error) => {
                error!("Error fetching data from message queue {:?}", error);
                // no error handling necessary - message won't be acked - it was never delivered properly
            }
        };
    }
    Ok(())
}

async fn new_producer(config: &Config) -> anyhow::Result<CommonPublisher> {
    Ok(match config.message_queue {
        MessageQueueKind::Kafka => {
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            CommonPublisher::new_kafka(brokers).await?
        }
        MessageQueueKind::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            CommonPublisher::new_amqp(connection_string).await?
        }
    })
}

async fn new_consumer(config: &Config, topic_or_queue: &str) -> anyhow::Result<CommonConsumer> {
    let config = match config.message_queue {
        MessageQueueKind::Kafka => {
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            let group_id = config
                .kafka_group_id
                .as_ref()
                .context("kafka group was not specified")?;
            CommonConsumerConfig::Kafka {
                brokers: &brokers,
                group_id: &group_id,
                topic: topic_or_queue,
            }
        }
        MessageQueueKind::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            let consumer_tag = config
                .amqp_consumer_tag
                .as_ref()
                .context("amqp consumer tag was not specified")?;
            CommonConsumerConfig::Amqp {
                connection_string: &connection_string,
                consumer_tag: &consumer_tag,
                queue_name: topic_or_queue,
                options: None,
            }
        }
    };
    Ok(CommonConsumer::new(config).await?)
}

async fn handle_message(
    message: Box<dyn CommunicationMessage>,
    cache: Arc<Mutex<LruCache<Uuid, String>>>,
    producer: Arc<CommonPublisher>,
    error_topic_or_exchange: Arc<String>,
    schema_registry_addr: Arc<String>,
) {
    let result: anyhow::Result<()> = async {
        let payload = message.payload()?;

        let insert_message: DataRouterInsertMessage =
            serde_json::from_str(payload).context("Payload deserialization failed")?;

        let topic_name =
            get_schema_topic(&cache, insert_message.schema_id, &schema_registry_addr).await?;

        let payload = BorrowedInsertMessage {
            object_id: insert_message.object_id,
            schema_id: insert_message.schema_id,
            order_group_id: insert_message.order_group_id,
            timestamp: current_timestamp(),
            data: insert_message.data,
        };

        let key = payload
            .order_group_id
            .map(|tag| tag.to_string().replace("-", "."))
            .unwrap_or_else(|| "unordered".to_string());
        trace!("send_message {:?} {:?} ", key, topic_name);
        send_message(
            producer.as_ref(),
            &topic_name,
            &key,
            serde_json::to_vec(&payload)?,
        )
        .await;
        Ok(())
    }
    .await;

    counter!("cdl.data-router.input-request", 1);

    if let Err(error) = result {
        error!("{:?}", error);
        send_message(
            producer.as_ref(),
            &error_topic_or_exchange,
            SERVICE_NAME,
            format!("{:?}", error).into(),
        )
        .await;
    }
    let ack_result = message.ack().await;
    if let Err(e) = ack_result {
        error!(
            "Fatal error, delivery status for message not received. {:?}",
            e
        );
        process::abort();
    }
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
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

use anyhow::Context;
use futures_util::stream::StreamExt;
use log::error;
use lru_cache::LruCache;
use schema_registry::{connect_to_registry, rpc::schema::Id};
use serde::{Deserialize, Serialize};
use std::{
    process,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use structopt::StructOpt;
use tokio::pin;
use utils::{
    abort_on_poison,
    message_types::{CommandServiceInsertMessage, DataRouterInputData},
    messaging_system::{
        consumer::CommonConsumer, message::CommunicationMessage, publisher::CommonPublisher,
    },
    metrics::{self, counter},
};
use uuid::Uuid;

const SERVICE_NAME: &str = "data-router";

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    #[structopt(long, env)]
    pub kafka_group_id: String,
    #[structopt(long, env)]
    pub kafka_topic: String,
    #[structopt(long, env)]
    pub kafka_brokers: String,
    #[structopt(long, env)]
    pub kafka_error_channel: String,
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

    let consumer = CommonConsumer::new_kafka(
        &config.kafka_group_id,
        &config.kafka_brokers,
        &[&config.kafka_topic],
    )
    .await?;
    let producer = Arc::new(
        CommonPublisher::new_kafka(&config.kafka_brokers)
            .await
            .unwrap(),
    );
    let cache = Arc::new(Mutex::new(LruCache::new(config.cache_capacity)));
    let consumer = consumer.leak();
    let message_stream = consumer.consume().await;
    pin!(message_stream);

    let kafka_error_channel = Arc::new(config.kafka_error_channel);
    let schema_registry_addr = Arc::new(config.schema_registry_addr);

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(message) => {
                tokio::spawn(handle_message(
                    message,
                    cache.clone(),
                    producer.clone(),
                    kafka_error_channel.clone(),
                    schema_registry_addr.clone(),
                ));
            }
            Err(error) => {
                error!("Error fetching data from message queue {:?}", error);
                // no error handling necessary - message won't be acked - it was never delivered properly
            }
        };
    }
    Ok(())
}

async fn handle_message(
    message: Box<dyn CommunicationMessage>,
    cache: Arc<Mutex<LruCache<Uuid, String>>>,
    producer: Arc<CommonPublisher>,
    kafka_error_channel: Arc<String>,
    schema_registry_addr: Arc<String>,
) {
    let result: anyhow::Result<()> = async {
        let event: DataRouterInputData =
            serde_json::from_str(message.payload()?).context("Payload deserialization failed")?;
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards"); // TODO: Ordering can be different when scaling
        let topic_name = get_schema_topic(&cache, &event, &schema_registry_addr).await?;
        let data = CommandServiceInsertMessage {
            object_id: event.object_id,
            payload: event.data,
            schema_id: event.schema_id,
            timestamp: since_the_epoch.as_millis() as i64, // TODO: Change types?
        };
        let payload = serde_json::to_vec(&data).unwrap_or_default();
        let key = data.object_id.to_string();
        send_message(producer.as_ref(), &topic_name, &key, payload).await;
        Ok(())
    }
    .await;

    counter!("cdl.data-router.input-request", 1);

    if let Err(error) = result {
        error!("{:?}", error);
        send_message(
            producer.as_ref(),
            &kafka_error_channel,
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
    event: &DataRouterInputData<'_>,
    schema_addr: &str,
) -> anyhow::Result<String> {
    let recv_channel = cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .get_mut(&event.schema_id)
        .cloned();
    if let Some(val) = recv_channel {
        return Ok(val);
    }

    let mut client = connect_to_registry(schema_addr.to_owned()).await?;
    let channel = client
        .get_schema_topic(Id {
            id: event.schema_id.to_string(),
        })
        .await?
        .into_inner()
        .topic;
    cache
        .lock()
        .unwrap_or_else(abort_on_poison)
        .insert(event.schema_id, channel.clone());

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

use std::sync::Arc;

use crate::{config::Config, error::Error, events::EventStream, events::EventSubscriber};
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Context {
    config: Arc<Config>,
    //                               Topic, Event stream
    kafka_events: Arc<Mutex<HashMap<String, EventSubscriber>>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(config: Arc<Config>) -> Self {
        Context {
            config,
            kafka_events: Default::default(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn connect_to_registry(&self) -> Result<SchemaRegistryConn, Error> {
        // TODO: Make proper connection pool
        let new_conn = rpc::schema_registry::connect(self.config.registry_addr.clone()).await?;
        Ok(new_conn)
    }

    pub async fn subscribe_on_kafka_topic(&self, topic: &str) -> Result<EventStream, Error> {
        log::debug!("subscribe on kafka topic {}", topic);
        self.consume_kafka_topic_inner(topic)
            .await
            .map_err(|e| Error::KafkaClientError(format!("{:?}", e)))
    }

    async fn consume_kafka_topic_inner(&self, topic: &str) -> Result<EventStream, anyhow::Error> {
        let mut event_map = self.kafka_events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => {
                let stream = subscriber.subscribe();
                Ok(stream)
            }
            None => {
                let (subscriber, stream) = EventSubscriber::new(&self.config.kafka, topic)?;
                event_map.insert(topic.to_string(), subscriber);
                Ok(stream)
            }
        }
    }
}

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

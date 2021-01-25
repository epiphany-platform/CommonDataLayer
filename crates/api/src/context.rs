use std::sync::Arc;

use crate::{config::Config, events::EventStream, events::EventSubscriber};
use anyhow::Result;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Context {
    config: Arc<Config>,
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

    pub async fn connect_to_registry(&self) -> Result<SchemaRegistryConn> {
        // TODO: Make proper connection pool
        let new_conn = rpc::schema_registry::connect(self.config.registry_addr.clone()).await?;
        Ok(new_conn)
    }

    pub async fn subscribe_on_kafka_topic(&self, topic: &str) -> Result<EventStream> {
        log::debug!("subscribe on kafka topic {}", topic);
        let mut event_map = self.kafka_events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => {
                let stream = subscriber.subscribe();
                Ok(stream)
            }
            None => {
                let kafka_events = self.kafka_events.clone();
                let (subscriber, stream) =
                    EventSubscriber::new(&self.config.kafka, topic, move |topic| async move {
                        log::warn!("Kafka stream has closed");
                        // Remove topic from hashmap so next time someone ask about this stream,
                        // it will be recreated
                        kafka_events.lock().await.remove(&topic);
                    })
                    .await?;
                event_map.insert(topic.into(), subscriber);
                Ok(stream)
            }
        }
    }
}

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

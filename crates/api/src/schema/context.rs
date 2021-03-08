use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use tokio::sync::Mutex;
use utils::communication::publisher::CommonPublisher;

use crate::{
    config::{CommunicationMethodConfig, Config},
    events::EventStream,
    events::EventSubscriber,
};

#[derive(Clone)]
pub struct Context {
    config: Arc<Config>,
    mq_events: Arc<Mutex<HashMap<String, EventSubscriber>>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(config: Arc<Config>) -> Self {
        Context {
            config,
            mq_events: Default::default(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn connect_to_registry(&self) -> Result<SchemaRegistryConn> {
        // TODO: Make proper connection pool
        let new_conn =
            rpc::schema_registry::connect(self.config.schema_registry_addr.clone()).await?;
        Ok(new_conn)
    }

    pub async fn subscribe_on_communication_method(&self, topic: &str) -> Result<EventStream> {
        log::debug!("subscribe on message queue: {}", topic);
        let mut event_map = self.mq_events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => {
                let stream = subscriber.subscribe();
                Ok(stream)
            }
            None => {
                let kafka_events = self.mq_events.clone();
                let (subscriber, stream) = EventSubscriber::new(
                    self.config.communication_method.config()?,
                    topic,
                    move |topic| async move {
                        log::warn!("Message queue stream has closed");
                        // Remove topic from hashmap so next time someone ask about this stream,
                        // it will be recreated
                        kafka_events.lock().await.remove(&topic);
                    },
                )
                .await?;
                event_map.insert(topic.into(), subscriber);
                Ok(stream)
            }
        }
    }

    pub async fn connect_to_cdl_input(&self) -> anyhow::Result<CommonPublisher> {
        match self.config().communication_method.config()? {
            CommunicationMethodConfig::Amqp {
                connection_string, ..
            } => CommonPublisher::new_amqp(&connection_string)
                .await
                .context("Unable to open RabbitMQ publisher for Ingestion Sink"),
            CommunicationMethodConfig::Kafka { brokers, .. } => {
                CommonPublisher::new_kafka(&brokers)
                    .await
                    .context("Unable to open Kafka publisher for Ingestion Sink")
            }
            CommunicationMethodConfig::Grpc => CommonPublisher::new_grpc("ingestion-sink")
                .await
                .context("Unable to create GRPC publisher for Ingestion Sink"),
        }
    }
}

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

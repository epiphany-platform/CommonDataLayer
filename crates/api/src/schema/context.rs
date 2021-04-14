use std::collections::HashMap;
use std::sync::Arc;

use bb8::{Pool, PooledConnection};

use crate::{config::Config, events::EventStream, events::EventSubscriber};
use rpc::edge_registry::edge_registry_client::EdgeRegistryClient;
use rpc::object_builder::object_builder_client::ObjectBuilderClient;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use tokio::sync::Mutex;

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type EdgeRegistryPool = Pool<EdgeRegistryConnectionManager>;
pub type ObjectBuilderPool = Pool<ObjectBuilderConnectionManager>;

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;
pub type EdgeRegistryConn = EdgeRegistryClient<Channel>;
pub type ObjectBuilderConn = ObjectBuilderClient<Channel>;

#[derive(Clone)]
pub struct MQEvents {
    pub events: Arc<Mutex<HashMap<String, EventSubscriber>>>,
}

pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub struct EdgeRegistryConnectionManager {
    pub address: String,
}

pub struct ObjectBuilderConnectionManager {
    pub address: String,
}

impl MQEvents {
    pub async fn subscribe_on_communication_method(
        &self,
        topic: &str,
        config: &Config,
    ) -> anyhow::Result<EventStream> {
        tracing::debug!("subscribe on message queue: {}", topic);

        let mut event_map = self.events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => {
                let stream = subscriber.subscribe();
                Ok(stream)
            }
            None => {
                let kafka_events = self.events.clone();
                let (subscriber, stream) = EventSubscriber::new(
                    config.communication_method.config()?,
                    topic,
                    move |topic| async move {
                        tracing::warn!("Message queue stream has closed");
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
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::schema_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.ping(rpc::schema_registry::Empty {})
            .await
            .map_err(rpc::error::schema_registry_error)?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl bb8::ManageConnection for EdgeRegistryConnectionManager {
    type Connection = EdgeRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::edge_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.heartbeat(rpc::edge_registry::Empty {})
            .await
            .map_err(rpc::error::edge_registry_error)?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[async_trait::async_trait]
impl bb8::ManageConnection for ObjectBuilderConnectionManager {
    type Connection = ObjectBuilderConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to object builder");

        rpc::object_builder::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.heartbeat(rpc::object_builder::Empty {})
            .await
            .map_err(rpc::error::object_builder_error)?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

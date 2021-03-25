use std::collections::HashMap;
use std::sync::Arc;

use bb8::Pool;
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use tokio::sync::Mutex;

use crate::{config::Config, events::EventStream, events::EventSubscriber};

#[derive(Clone)]
pub struct MQEvents {
    pub events: Arc<Mutex<HashMap<String, EventSubscriber>>>,
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

pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::schema_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        conn.heartbeat(rpc::schema_registry::Empty {})
            .await
            .map_err(rpc::error::registry_error)?;

        Ok(conn)
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

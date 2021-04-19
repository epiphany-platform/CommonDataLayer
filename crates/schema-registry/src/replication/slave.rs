use super::{CommunicationMethod, ReplicationEvent, ReplicationMethodConfig};
use crate::db::SchemaDb;
use crate::error::{RegistryError, RegistryResult};
use async_trait::async_trait;
use std::{process, sync::Arc};
use tokio::sync::oneshot::Receiver;
use tracing::{error, trace};
use utils::communication::{
    consumer::CommonConsumer,
    consumer::{CommonConsumerConfig, ConsumerHandler},
    message::CommunicationMessage,
};

struct Handler {
    kill_signal: Receiver<()>,
    db: Arc<SchemaDb>,
}

#[async_trait]
impl ConsumerHandler for Handler {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        if self.kill_signal.try_recv().is_ok() {
            anyhow::bail!("Slave replication disabled");
        };
        trace!("Received message");
        if let Err(error) = consume_message(msg, &self.db).await {
            error!("Error while processing message: {}", error);
        }
        Ok(())
    }
}

pub async fn consume_mq(
    config: ReplicationMethodConfig,
    db: Arc<SchemaDb>,
    kill_signal: Receiver<()>,
) -> RegistryResult<()> {
    let config = match &config.queue {
        CommunicationMethod::Kafka(kafka) => CommonConsumerConfig::Kafka {
            group_id: &kafka.group_id,
            brokers: &kafka.brokers,
            topic: &config.source,
        },
        CommunicationMethod::Amqp(amqp) => CommonConsumerConfig::Amqp {
            connection_string: &amqp.connection_string,
            consumer_tag: &amqp.consumer_tag,
            queue_name: &config.source,
            options: None,
        },
    };
    let consumer = CommonConsumer::new(config).await.unwrap_or_else(|err| {
        error!(
            "Fatal error. Encountered some problems connecting to kafka service. {:?}",
            err
        );
        process::abort();
    });
    consumer.run(Handler { kill_signal, db }).await?;

    Ok(())
}

async fn consume_message(
    message: &'_ dyn CommunicationMessage,
    db: &SchemaDb,
) -> RegistryResult<()> {
    let payload = message.payload()?;
    let event: ReplicationEvent =
        serde_json::from_str(payload).map_err(RegistryError::SerdeError)?;
    trace!("Consuming message: {:?}", event);
    match event {
        ReplicationEvent::AddSchema { id, schema } => {
            db.add_schema(schema, Some(id))?;
        }
        ReplicationEvent::AddSchemaVersion { id, new_version } => {
            db.add_new_version_of_schema(id, new_version)?;
        }
        ReplicationEvent::AddViewToSchema {
            schema_id,
            view,
            view_id,
        } => {
            db.add_view_to_schema(schema_id, view, Some(view_id))?;
        }
        ReplicationEvent::UpdateSchemaMetadata {
            id,
            name,
            insert_destination,
            query_address,
            schema_type,
        } => {
            if let Some(name) = name {
                db.update_schema_name(id, name)?;
            }
            if let Some(insert_destination) = insert_destination {
                db.update_schema_insert_destination(id, insert_destination)?;
            }
            if let Some(query_address) = query_address {
                db.update_schema_query_address(id, query_address)?;
            }
            if let Some(schema_type) = schema_type {
                db.update_schema_type(id, schema_type)?;
            }
        }
        ReplicationEvent::UpdateView { id, view } => {
            db.update_view(id, view)?;
        }
    };

    Ok(())
}

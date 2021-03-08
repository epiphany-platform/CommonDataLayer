use super::{CommunicationMethod, ReplicationEvent, ReplicationMethodConfig};
use crate::db::SchemaDb;
use anyhow::Context;
use async_trait::async_trait;
use log::{error, trace};
use std::{process, sync::Arc};
use tokio::sync::oneshot::Receiver;
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
) -> anyhow::Result<()> {
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
    consumer.run(Handler { db, kill_signal }).await?;

    Ok(())
}

async fn consume_message(
    message: &'_ dyn CommunicationMessage,
    db: &SchemaDb,
) -> anyhow::Result<()> {
    let payload = message.payload()?;
    let event: ReplicationEvent =
        serde_json::from_str(payload).context("Payload deserialization failed")?;
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
            topic,
            query_address,
            schema_type,
        } => {
            if let Some(name) = name {
                db.update_schema_name(id, name)?;
            }
            if let Some(topic) = topic {
                db.update_schema_topic(id, topic)?;
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

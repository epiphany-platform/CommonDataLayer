use super::{KafkaConfig, ReplicationEvent};
use crate::db::SchemaDb;
use anyhow::Context;
use log::{error, info, trace};
use std::{process, sync::Arc};
use tokio::stream::StreamExt;
use tokio::{pin, sync::oneshot::Receiver};
use utils::messaging_system::{consumer::CommonConsumer, message::CommunicationMessage};

pub async fn consume_kafka_topic(
    config: KafkaConfig,
    db: Arc<SchemaDb>,
    mut kill_signal: Receiver<()>,
) -> anyhow::Result<()> {
    let topics: Vec<_> = config.topics.iter().map(String::as_str).collect();
    let mut consumer =
        CommonConsumer::new_kafka(&config.group_id, &config.brokers, topics.as_slice())
            .await
            .unwrap_or_else(|err| {
                error!(
                    "Fatal error. Encountered some problems connecting to kafka service. {:?}",
                    err
                );
                process::abort();
            });
    let message_stream = consumer.consume().await;
    pin!(message_stream);
    while let Some(message) = message_stream.next().await {
        if kill_signal.try_recv().is_ok() {
            info!("Slave replication disabled");
            return Ok(());
        };
        trace!("Received message");
        match message {
            Err(error) => error!("Received malformed message: {}", error),
            Ok(message) => {
                if let Err(error) = consume_message(message.as_ref(), &db).await {
                    error!("Error while processing message: {}", error);
                }
            }
        }
    }

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

    message.ack().await?;
    Ok(())
}

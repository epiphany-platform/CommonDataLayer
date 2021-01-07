use super::{KafkaConfig, ReplicationEvent};
use log::{error, info};
use std::{
    process,
    sync::{mpsc, Arc},
};
use tokio::{runtime::Handle, sync::oneshot};
use utils::messaging_system::publisher::CommonPublisher;

pub async fn replicate_db_events(
    config: KafkaConfig,
    recv: mpsc::Receiver<ReplicationEvent>,
    tokio_runtime: Handle,
    mut kill_signal: oneshot::Receiver<()>,
) {
    let producer = CommonPublisher::new_kafka(&config.brokers)
        .await
        .unwrap_or_else(|_e| {
            error!("Fatal error, synchronization channel cannot be created.");
            process::abort();
        });
    let producer = Arc::new(producer);
    loop {
        let event = recv.recv().unwrap_or_else(|_e| {
            error!("Fatal error, synchronization channel closed.");
            process::abort();
        });
        if kill_signal.try_recv().is_ok() {
            info!("Master replication disabled");
            return;
        };

        tokio_runtime
            .enter(|| send_messages_to_kafka(producer.clone(), config.topics[0].clone(), event));
    }
}

fn send_messages_to_kafka(
    producer: Arc<CommonPublisher>,
    topic_name: String,
    event: ReplicationEvent,
) {
    let key = match &event {
        ReplicationEvent::AddSchema { id, .. } => id,
        ReplicationEvent::AddSchemaVersion { id, .. } => id,
        ReplicationEvent::AddViewToSchema { schema_id, .. } => schema_id,
        ReplicationEvent::UpdateSchemaMetadata { id, .. } => id,
        ReplicationEvent::UpdateView { id, .. } => id,
    };
    let serialized = serde_json::to_string(&event).unwrap();
    let serialized_key = key.to_string();
    tokio::spawn(async move {
        let delivery_status =
            producer.publish_message(&topic_name, &serialized_key, serialized.as_bytes().to_vec());
        if delivery_status.await.is_err() {
            error!("Fatal error, delivery status for message not received.");
            process::abort();
        }
    });
}

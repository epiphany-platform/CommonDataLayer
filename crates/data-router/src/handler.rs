use anyhow::Context;
use async_trait::async_trait;
use cdl_dto::ingestion::{BorrowedInsertMessage, DataRouterInsertMessage};
use communication_utils::{
    get_order_group_id, message::CommunicationMessage, parallel_consumer::ParallelConsumerHandler,
    publisher::CommonPublisher,
};
use metrics_utils::{self as metrics, counter};
use misc_utils::current_timestamp;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, trace};
use utils::parallel_task_queue::ParallelTaskQueue;

pub struct Handler {
    pub producer: Arc<CommonPublisher>,
    pub schema_registry_addr: Arc<String>,
    pub task_queue: Arc<ParallelTaskQueue>,
}

#[async_trait]
impl ParallelConsumerHandler for Handler {
    #[tracing::instrument(skip(self, message))]
    async fn handle<'a>(&'a self, message: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let order_group_id = get_order_group_id(message);
        let _guard =
            order_group_id.map(move |x| async move { self.task_queue.acquire_permit(x).await });

        trace!(
            "Received message ({:?}) `{:?}`",
            message.key(),
            message.payload()
        );

        let message_key = get_order_group_id(message).unwrap_or_default();
        counter!("cdl.data-router.input-msg", 1);
        let result = async {
            let json_something: Value = serde_json::from_str(message.payload()?)
                .context("Payload deserialization failed")?;
            if json_something.is_array() {
                trace!("Processing multimessage");

                let maybe_array: Vec<DataRouterInsertMessage> = serde_json::from_str(
                    message.payload()?,
                )
                .context("Payload deserialization failed, message is not a valid cdl message ")?;

                let mut result = Ok(());

                for entry in maybe_array.iter() {
                    let r = route(
                        entry,
                        &message_key,
                        &self.producer,
                        &self.schema_registry_addr,
                    )
                    .await
                    .context("Tried to send message and failed");

                    counter!("cdl.data-router.input-multimsg", 1);
                    counter!("cdl.data-router.processed", 1);

                    if r.is_err() {
                        result = r;
                    }
                }

                result
            } else {
                trace!("Processing single message");

                let owned: DataRouterInsertMessage =
                    serde_json::from_str::<DataRouterInsertMessage>(message.payload()?).context(
                        "Payload deserialization failed, message is not a valid cdl message",
                    )?;
                let result = route(
                    &owned,
                    &message_key,
                    &self.producer,
                    &self.schema_registry_addr,
                )
                .await
                .context("Tried to send message and failed");
                counter!("cdl.data-router.input-singlemsg", 1);
                counter!("cdl.data-router.processed", 1);

                result
            }
        }
        .await;

        counter!("cdl.data-router.input-request", 1);

        if let Err(error) = result {
            counter!("cdl.data-router.error", 1);

            return Err(error);
        } else {
            counter!("cdl.data-router.success", 1);
        }

        Ok(())
    }
}

#[tracing::instrument(skip(producer))]
async fn route(
    event: &DataRouterInsertMessage<'_>,
    message_key: &str,
    producer: &CommonPublisher,
    schema_registry_addr: &str,
) -> anyhow::Result<()> {
    let payload = BorrowedInsertMessage {
        object_id: event.object_id,
        schema_id: event.schema_id,
        timestamp: current_timestamp(),
        data: event.data,
    };

    let mut conn = rpc::schema_registry::connect(schema_registry_addr.to_owned()).await?;
    let schema = conn
        .get_schema_metadata(rpc::schema_registry::Id {
            id: event.schema_id.to_string(),
        })
        .await
        .context("failed to get schema metadata")?
        .into_inner();

    send_message(
        producer,
        &schema.insert_destination,
        message_key,
        serde_json::to_vec(&payload)?,
    )
    .await;
    Ok(())
}

#[tracing::instrument(skip(producer))]
async fn send_message(
    producer: &CommonPublisher,
    insert_destination: &str,
    key: &str,
    payload: Vec<u8>,
) {
    let payload_len = payload.len();
    let delivery_status = producer
        .publish_message(&insert_destination, key, payload)
        .await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Insert destination: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            insert_destination, key, payload_len, delivery_status
        );
    } else {
        counter!("cdl.data-router.output-singleok", 1);
    }
}

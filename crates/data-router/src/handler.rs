use anyhow::{bail, Context};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, trace};
use uuid::Uuid;

use cache::DynamicCache;
use cdl_dto::ingestion::{BorrowedInsertMessage, DataRouterInsertMessage};
use communication_utils::{
    get_order_group_id, message::CommunicationMessage, parallel_consumer::ParallelConsumerHandler,
    publisher::CommonPublisher,
};
use lenient_semver::Version;
use metrics_utils::{self as metrics, counter};
use misc_utils::current_timestamp;
use settings_utils::RepositoryStaticRouting;
use std::collections::HashMap;
use utils::parallel_task_queue::ParallelTaskQueue;

static CDL_INPUT_PROTOCOL_VERSION_MAJOR: u64 = 1;
static CDL_INPUT_PROTOCOL_VERSION_MINOR: u64 = 0;

pub struct Handler {
    pub cache: DynamicCache<Uuid, String>,
    pub producer: Arc<CommonPublisher>,
    pub task_queue: Arc<ParallelTaskQueue>,
    pub routing_table: Arc<HashMap<String, RepositoryStaticRouting>>,
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
                    let r = if let Some(repository_id) = &entry.options.repository_id {
                        if let Some(routing) = self.routing_table.get(repository_id) {
                            route_static(
                                entry,
                                &message_key,
                                &self.producer,
                                &routing.insert_destination,
                            )
                            .await
                        } else {
                            Err(anyhow::Error::msg("No such entry in routing table"))
                        }
                    } else {
                        route(&self.cache, entry, &message_key, &self.producer)
                            .await
                            .context("Tried to send message and failed")
                    };

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

                let result = if let Some(repository_id) = &owned.options.repository_id {
                    if let Some(routing) = self.routing_table.get(repository_id) {
                        route_static(
                            &owned,
                            &message_key,
                            &self.producer,
                            &routing.insert_destination,
                        )
                        .await
                    } else {
                        Err(anyhow::Error::msg("No such entry in routing table"))
                    }
                } else {
                    route(&self.cache, &owned, &message_key, &self.producer)
                        .await
                        .context("Tried to send message and failed")
                };
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

fn check_version_matrix(version: lenient_semver::Version) -> anyhow::Result<()> {
    if version.major != CDL_INPUT_PROTOCOL_VERSION_MAJOR {
        bail!("Unsupported protocol : major version")
    }
    if version.minor != CDL_INPUT_PROTOCOL_VERSION_MINOR {
        bail!("Unsupported protocol : minor version")
    }
    Ok(())
}

fn check_inbound_version(string: &str) -> anyhow::Result<()> {
    let version =
        lenient_semver::parse_into::<Version>(string).map_err(|e| anyhow::anyhow!("{}", e))?;

    let (version, pre, build) = version.disassociate_metadata();

    let pre: Vec<String> = pre.into_iter().map(ToOwned::to_owned).collect();
    let build: Vec<String> = build.into_iter().map(ToOwned::to_owned).collect();

    if !pre.is_empty() {
        anyhow::bail!("Malformed message, version can not contain prerelease part");
    }
    if !build.is_empty() {
        anyhow::bail!("Malformed message, version can not contain build part");
    }
    if version.patch != 0 {
        anyhow::bail!("Malformed message, version can not contain patch part");
    }
    check_version_matrix(version)?;
    Ok(())
}

#[tracing::instrument(skip(publisher))]
async fn route_static(
    event: &DataRouterInsertMessage<'_>,
    key: &str,
    publisher: &CommonPublisher,
    repository_path: &str,
) -> anyhow::Result<()> {
    check_inbound_version(&event.version)?;
    let payload = BorrowedInsertMessage {
        object_id: event.object_id,
        schema_id: event.schema_id,
        timestamp: current_timestamp(),
        data: event.data,
    };

    send_message(
        publisher,
        repository_path,
        key,
        serde_json::to_vec(&payload)?,
    )
    .await;

    Ok(())
}

#[tracing::instrument(skip(cache, publisher))]
async fn route(
    cache: &DynamicCache<Uuid, String>,
    event: &DataRouterInsertMessage<'_>,
    key: &str,
    publisher: &CommonPublisher,
) -> anyhow::Result<()> {
    let insert_destination = cache.get(event.schema_id).await?;

    route_static(event, &key, publisher, &insert_destination).await
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

#[cfg(test)]
mod tests {
    use super::check_inbound_version;

    static PROPER_VERSION1: &str = "1.0";
    static PROPER_VERSION2: &str = "1";
    static IMPROPER_VERSION1: &str = "1.0-rc1";
    static IMPROPER_VERSION2: &str = "1.0.1";
    static IMPROPER_VERSION3: &str = "0.1.0";
    static IMPROPER_VERSION4: &str = "0.1-rc1";
    static IMPROPER_VERSION5: &str = "1.1.1.1";
    static IMPROPER_VERSION6: &str = "darkside";

    #[test]
    fn test_version_checking_green() {
        assert!(check_inbound_version(PROPER_VERSION1).is_ok());
        assert!(check_inbound_version(PROPER_VERSION2).is_ok());
    }

    #[test]
    fn test_version_checking_red() {
        assert!(check_inbound_version(IMPROPER_VERSION1).is_err());
        assert!(check_inbound_version(IMPROPER_VERSION2).is_err());
        assert!(check_inbound_version(IMPROPER_VERSION3).is_err());
        assert!(check_inbound_version(IMPROPER_VERSION4).is_err());
        assert!(check_inbound_version(IMPROPER_VERSION5).is_err());
        assert!(check_inbound_version(IMPROPER_VERSION6).is_err());
    }
}

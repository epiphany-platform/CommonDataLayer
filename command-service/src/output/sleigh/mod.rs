use crate::communication::resolution::Resolution;
use crate::output::sleigh::connection_pool::SleighConnectionManager;
use crate::output::OutputPlugin;
use bb8::Pool;
pub use config::SleighOutputConfig;
pub use error::Error;
use log::{error, trace};
use rpc::document_storage::StoreRequest;
use utils::message_types::BorrowedInsertMessage;
use utils::metrics::counter;

mod connection_pool;

pub mod config;
pub mod error;

pub struct SleighOutputPlugin {
    pool: Pool<SleighConnectionManager>,
}

impl SleighOutputPlugin {
    pub async fn new(config: SleighOutputConfig) -> Result<Self, Error> {
        Ok(Self {
            pool: Pool::builder()
                .max_size(config.pool_size)
                .build(SleighConnectionManager { addr: config.addr })
                .await
                .map_err(Error::FailedToConnect)?,
        })
    }
}

#[async_trait::async_trait]
impl OutputPlugin for SleighOutputPlugin {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let mut connection = match self.pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to get connection from pool {:?}", err);
                return Resolution::CommandServiceFailure;
            }
        };

        trace!("Storing message {:?}", msg);

        match connection
            .store(StoreRequest {
                object_id: msg.object_id.to_string(),
                schema_id: msg.schema_id.to_string(),
                data: msg.data.get().as_bytes().to_vec(),
            })
            .await
        {
            Ok(_) => {
                counter!("cdl.command-service.store.sleigh", 1);

                Resolution::Success
            }
            Err(err) => Resolution::StorageLayerFailure {
                description: err.to_string(),
            },
        }
    }

    fn name(&self) -> &'static str {
        "Sleigh datastore"
    }
}

use crate::communication::resolution::Resolution;
use crate::communication::GenericMessage;
use crate::output::sleigh::connection_pool::SleighConnectionManager;
use crate::output::OutputPlugin;
use bb8::{Pool, PooledConnection};
pub use config::SleighOutputConfig;
use document_storage::grpc::schema::StoreRequest;
pub use error::Error;
use log::{error, trace};
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

    async fn store_message(
        msg: GenericMessage,
        mut client: PooledConnection<'_, SleighConnectionManager>,
    ) -> Resolution {
        trace!("Storing message {:?}", msg);

        match client
            .store(StoreRequest {
                object_id: msg.object_id.to_string(),
                schema_id: msg.schema_id.to_string(),
                data: msg.payload,
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
}

#[async_trait::async_trait]
impl OutputPlugin for SleighOutputPlugin {
    async fn handle_message(&self, msg: GenericMessage) -> Resolution {
        let resolution = match self.pool.get().await {
            Ok(client) => SleighOutputPlugin::store_message(msg, client).await,
            Err(err) => {
                error!("Failed to get connection from pool {:?}", err);
                Resolution::CommandServiceFailure
            }
        };

        resolution
    }

    fn name(&self) -> &'static str {
        "Sleigh datastore"
    }
}

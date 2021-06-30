use cache::{CacheSupplier, DynamicCache};
use rpc::schema_registry::types::SchemaType;
use std::convert::TryInto;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type SchemaCache = Mutex<DynamicCache<Uuid, (String, SchemaType)>>;

pub struct SchemaMetadataSupplier {
    schema_registry_url: String,
}

impl SchemaMetadataSupplier {
    pub fn boxed(schema_registry_url: String) -> Box<Self> {
        Box::new(Self {
            schema_registry_url,
        })
    }
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, (String, SchemaType)> for SchemaMetadataSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<(String, SchemaType)> {
        let mut conn = rpc::schema_registry::connect(self.schema_registry_url.clone()).await?;

        let metadata = conn
            .get_schema_metadata(rpc::schema_registry::Id {
                id: key.to_string(),
            })
            .await?
            .into_inner();

        Ok((metadata.query_address, metadata.schema_type.try_into()?))
    }
}

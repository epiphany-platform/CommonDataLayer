use cache::{CacheSupplier, DynamicCache};
use rpc::schema_registry::types::SchemaType;
use std::convert::TryInto;
use uuid::Uuid;

pub type SchemaCache = DynamicCache<Uuid, SchemaMetadata>;

pub struct SchemaMetadataSupplier {
    schema_registry_url: String,
}

#[derive(Clone, Debug)]
pub struct SchemaMetadata {
    pub query_address: String,
    pub schema_type: SchemaType,
}

impl SchemaMetadataSupplier {
    pub fn boxed(schema_registry_url: String) -> Box<Self> {
        Box::new(Self {
            schema_registry_url,
        })
    }
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, SchemaMetadata> for SchemaMetadataSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<SchemaMetadata> {
        let mut conn = rpc::schema_registry::connect(self.schema_registry_url.clone()).await?;

        let metadata = conn
            .get_schema_metadata(rpc::schema_registry::Id {
                id: key.to_string(),
            })
            .await?
            .into_inner();

        Ok(SchemaMetadata {
            query_address: metadata.query_address,
            schema_type: metadata.schema_type.try_into()?,
        })
    }
}

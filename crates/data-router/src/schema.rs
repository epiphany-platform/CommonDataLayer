use cache::{CacheSupplier, DynamicCache};
use rpc::schema_registry::Id;
use uuid::Uuid;

pub type SchemaCache = DynamicCache<SchemaMetadataSupplier, Uuid, String>;

pub struct SchemaMetadataSupplier {
    schema_registry_url: String,
}

impl SchemaMetadataSupplier {
    pub fn new(schema_registry_url: String) -> Self {
        Self {
            schema_registry_url,
        }
    }
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, String> for SchemaMetadataSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<String> {
        let mut client = rpc::schema_registry::connect(self.schema_registry_url.to_owned()).await?;

        Ok(client
            .get_schema_metadata(Id {
                id: key.to_string(),
            })
            .await?
            .into_inner()
            .insert_destination)
    }
}

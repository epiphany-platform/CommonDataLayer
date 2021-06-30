use cache::CacheSupplier;
use rpc::schema_registry::Id;
use uuid::Uuid;

pub struct SchemaMetadataFetcher {
    schema_registry_url: String,
}

impl SchemaMetadataFetcher {
    pub fn boxed(schema_registry_url: String) -> Box<Self> {
        Box::new(Self {
            schema_registry_url,
        })
    }
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, String> for SchemaMetadataFetcher {
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

use cache::{CacheSupplier, DynamicCache};
use cdl_dto::materialization::FullView;
use cdl_dto::TryFromRpc;
use rpc::schema_registry::Id;
use uuid::Uuid;

pub type ViewCache = DynamicCache<ViewSupplier, Uuid, FullView>;

pub struct ViewSupplier {
    schema_registry_url: String,
}

impl ViewSupplier {
    pub fn new(schema_registry_url: String) -> Self {
        Self {
            schema_registry_url,
        }
    }
}

#[async_trait::async_trait]
impl CacheSupplier<Uuid, FullView> for ViewSupplier {
    async fn retrieve(&self, key: Uuid) -> anyhow::Result<FullView> {
        let mut client = rpc::schema_registry::connect(self.schema_registry_url.to_owned()).await?;
        let view_definition = client
            .get_view(Id {
                id: key.to_string(),
            })
            .await?
            .into_inner();

        Ok(FullView::try_from_rpc(view_definition)?)
    }
}

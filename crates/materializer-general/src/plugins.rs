use cdl_dto::materialization::FullView;
use rpc::materializer_general::MaterializedView;
use serde_json::Value;

#[async_trait::async_trait]
pub trait MaterializerPlugin: Send + Sync {
    fn validate_options(&self, options: Value) -> anyhow::Result<()>;
    async fn upsert_view(
        &self,
        view: MaterializedView,
        view_definition: FullView,
    ) -> anyhow::Result<()>;
}

mod postgres;

pub use postgres::PostgresMaterializer;

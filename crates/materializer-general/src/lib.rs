pub mod args;
mod plugins;

use crate::args::MaterializerArgs;
use plugins::{MaterializerPlugin, PostgresMaterializer};
use rpc::materializer_general::MaterializedView;
use rpc::materializer_general::{general_materializer_server::GeneralMaterializer, Empty, Options};

pub struct MaterializerImpl {
    materializer: std::sync::Arc<dyn MaterializerPlugin>,
}

impl MaterializerImpl {
    pub async fn new(args: &MaterializerArgs) -> anyhow::Result<Self> {
        let materializer = match args {
            MaterializerArgs::Postgres(ref args) => {
                std::sync::Arc::new(PostgresMaterializer::new(args).await?)
            }
        };

        Ok(Self { materializer })
    }
}

impl MaterializerImpl {
    fn validate_options_inner(&self, options: &str) -> anyhow::Result<()> {
        let options: serde_json::Value = serde_json::from_str(&options)?;
        self.materializer.validate_options(options)?;
        Ok(())
    }
}

#[tonic::async_trait]
impl GeneralMaterializer for MaterializerImpl {
    #[tracing::instrument(skip(self))]
    async fn validate_options(
        &self,
        request: tonic::Request<Options>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let options: Options = request.into_inner();
        tracing::debug!(?options, "Options to validate");

        match self.validate_options_inner(&options.options) {
            Ok(_) => Ok(tonic::Response::new(Empty {})),
            Err(err) => Err(tonic::Status::invalid_argument(format!("{}", err))),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn upsert_view(
        &self,
        request: tonic::Request<MaterializedView>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let materialized_view = request.into_inner();
        tracing::debug!(?materialized_view, "materialized view");

        match self.materializer.upsert_view(materialized_view).await {
            Ok(_) => Ok(tonic::Response::new(Empty {})),
            Err(err) => Err(tonic::Status::internal(format!("{}", err))),
        }
    }
}

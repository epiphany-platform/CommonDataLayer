pub mod settings;

use std::pin::Pin;

use futures::Stream;
use rpc::{
    common::RowDefinition,
    materializer_ondemand::on_demand_materializer_server::OnDemandMaterializer,
    materializer_ondemand::Empty, object_builder::View,
};

pub struct MaterializerImpl {
    object_builder_addr: String, //TODO: Replace by bb8
}

impl MaterializerImpl {
    pub async fn new(object_builder_addr: &str) -> anyhow::Result<Self> {
        Ok(Self {
            object_builder_addr: object_builder_addr.to_string(),
        })
    }
}

#[tonic::async_trait]
impl OnDemandMaterializer for MaterializerImpl {
    type MaterializeStream =
        Pin<Box<dyn Stream<Item = Result<RowDefinition, tonic::Status>> + 'static + Send + Sync>>;

    #[tracing::instrument(skip(self))]
    async fn materialize(
        &self,
        request: tonic::Request<rpc::materializer_ondemand::OnDemandRequest>,
    ) -> Result<tonic::Response<Self::MaterializeStream>, tonic::Status> {
        utils::tracing::grpc::set_parent_span(&request);
        let request = request.into_inner();

        tracing::debug!(?request, "Handling");

        let view_id = request.view_id;
        let schemas = request
            .schemas
            .into_iter()
            .map(|(k, v)| (k, into_object_builder_schemas(v)))
            .collect();

        let stream = rpc::object_builder::connect(self.object_builder_addr.as_str())
            .await
            .map_err(|e| tonic::Status::internal(format!("{}", e)))?
            .materialize(utils::tracing::grpc::inject_span(View { view_id, schemas }))
            .await
            .map_err(|e| tonic::Status::internal(format!("{}", e)))?;

        let stream = Box::pin(stream.into_inner());

        Ok(tonic::Response::new(stream))
    }

    #[tracing::instrument(skip(self))]
    async fn heartbeat(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        //empty
        Ok(tonic::Response::new(Empty {}))
    }
}

// TODO?
fn into_object_builder_schemas(
    schemas: rpc::materializer_ondemand::Schema,
) -> rpc::object_builder::Schema {
    rpc::object_builder::Schema {
        object_ids: schemas.object_ids,
    }
}

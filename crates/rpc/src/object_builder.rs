use crate::error::ClientError;
use object_builder_client::ObjectBuilderClient;
use tonic::transport::Channel;

pub use crate::codegen::object_builder::*;

pub async fn connect(addr: impl Into<String>) -> Result<ObjectBuilderClient<Channel>, ClientError> {
    connect_inner(addr.into())
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "object builder",
            source: err,
        })
}

async fn connect_inner(
    addr: String,
) -> Result<ObjectBuilderClient<Channel>, tonic::transport::Error> {
    let conn = tonic::transport::Endpoint::new(addr)?.connect().await?;

    Ok(ObjectBuilderClient::with_interceptor(
        conn,
        tracing_tools::grpc::interceptor(),
    ))
}

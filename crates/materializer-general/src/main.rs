#![feature(async_closure)]

use materializer_general::{settings::Settings, MaterializerImpl};
use rpc::materializer_general::general_materializer_server::GeneralMaterializerServer;
use settings_utils::load_settings;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    utils::status_endpoints::serve(&settings.monitoring);
    metrics_utils::serve(&settings.monitoring);

    let notification_publisher = settings
        .notifications
        .publisher(
            async || settings.publisher().await,
            "Kafka".to_string(),
            "MaterializerGeneral",
        )
        .await?;

    let materializer = MaterializerImpl::new(
        settings.postgres,
        Arc::new(Mutex::new(notification_publisher)),
    )
    .await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(GeneralMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}

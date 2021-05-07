use materializer_general::{settings::Settings, MaterializerImpl};
use rpc::materializer_general::general_materializer_server::GeneralMaterializerServer;
use tonic::transport::Server;
use utils::settings::load_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    ::utils::tracing::init(settings.log.rust_log.as_str())?;

    tracing::debug!(?settings, "command-line arguments");

    utils::status_endpoints::serve(&settings.monitoring);
    utils::metrics::serve(&settings.monitoring);

    let materializer = MaterializerImpl::new(settings.postgres).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .trace_fn(utils::tracing::grpc::trace_fn)
        .add_service(GeneralMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}

use materializer_general::{settings::Settings, MaterializerImpl};
use rpc::materializer_general::general_materializer_server::GeneralMaterializerServer;
use tonic::transport::Server;
use utils::settings::load_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let settings: Settings = load_settings()?;

    utils::status_endpoints::serve(&settings.monitoring);
    utils::metrics::serve(&settings.monitoring);

    let materializer = MaterializerImpl::new(settings.postgres).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(GeneralMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}

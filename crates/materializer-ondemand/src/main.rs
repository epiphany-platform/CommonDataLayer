use materializer_ondemand::{settings::Settings, MaterializerImpl};
use rpc::materializer_ondemand::on_demand_materializer_server::OnDemandMaterializerServer;
use tonic::transport::Server;
use utils::settings::load_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    settings.log.init()?;

    tracing::debug!(?settings, "command-line arguments");

    utils::status_endpoints::serve(&settings.monitoring);
    utils::metrics::serve(&settings.monitoring);

    let materializer = MaterializerImpl::new(&settings.services.object_builder_url).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(OnDemandMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    Ok(())
}

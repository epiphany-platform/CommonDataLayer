use clap::Clap;
use materializer_ondemand::{args::Args, MaterializerImpl};
use rpc::materializer_ondemand::on_demand_materializer_server::OnDemandMaterializerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let args: Args = Args::parse();

    tracing::debug!(?args, "command-line arguments");

    utils::status_endpoints::serve(args.status_port);
    utils::metrics::serve(args.metrics_port);

    let materializer = MaterializerImpl::new(&args).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(OnDemandMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], args.input_port).into())
        .await?;

    Ok(())
}

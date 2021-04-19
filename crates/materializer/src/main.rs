use clap::Clap;
use materializer::{args::Args, MaterializerImpl};
use rpc::materializer::materializer_server::MaterializerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let args: Args = Args::parse();

    tracing::debug!(?args, "command-line arguments");

    utils::status_endpoints::serve(args.status_port);
    utils::metrics::serve(args.metrics_port);

    let materializer = MaterializerImpl::new(&args.materializer).await?;

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(MaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], args.input_port).into())
        .await?;

    Ok(())
}

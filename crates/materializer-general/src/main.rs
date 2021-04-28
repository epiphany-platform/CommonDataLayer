use clap::Clap;
use materializer_general::{args::Args, MaterializerImpl};
use rpc::materializer_general::general_materializer_server::GeneralMaterializerServer;
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
        .trace_fn(utils::tracing::grpc::trace_fn)
        .add_service(GeneralMaterializerServer::new(materializer))
        .serve(([0, 0, 0, 0], args.input_port).into())
        .await?;

    Ok(())
}

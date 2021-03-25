use args::Args;
use service::Service;
use structopt::StructOpt;

mod args;

mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let args: Args = Args::from_args();

    tracing::debug!(?args, "command-line arguments");

    utils::metrics::serve(args.metrics_port);

    Service::new(args.communication_config()?, &args.schema_registry_addr)
        .await?
        .listen()
        .await?;

    Ok(())
}

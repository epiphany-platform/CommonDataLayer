pub mod args;

use crate::args::Args;
use blob_store::grpc;
use log::{info, trace};
use structopt::StructOpt;
use utils::metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::from_args();

    trace!("Environment: {:?}", args);

    info!("Starting SleighDB");

    metrics::serve();
    grpc::run(args.datastore_root).await
}

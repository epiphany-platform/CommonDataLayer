use clap::Clap;
use object_builder::{args::Args, ObjectBuilderImpl};
use rpc::object_builder::object_builder_server::ObjectBuilderServer;
use tonic::transport::Server;
use utils::communication::consumer::CommonConsumer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let args: Args = Args::parse();

    tracing::debug!(?args, "command-line arguments");

    utils::status_endpoints::serve(args.status_port);
    utils::metrics::serve(args.metrics_port);

    let object_builder = ObjectBuilderImpl::new(&args).await?;
    if let Some(consumer_config) = args.consumer_config()? {
        let consumer = CommonConsumer::new(consumer_config).await?;
        let handler = object_builder.clone();
        tokio::spawn(async {
            tracing::info!("Listening for messages via MQ");

            match consumer.run(handler).await {
                Ok(_) => {
                    tracing::error!("MQ consumer finished work");
                }
                Err(err) => {
                    tracing::error!("MQ consumer returned with error: {:?}", err);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            std::process::abort();
        });
    }

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(ObjectBuilderServer::new(object_builder))
        .serve(([0, 0, 0, 0], args.input_port).into())
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

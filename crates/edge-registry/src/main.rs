use edge_registry::{EdgeRegistryImpl, RegistryConfig};
use log::*;
use rpc::edge_registry::edge_registry_server::EdgeRegistryServer;
use std::process;
use structopt::StructOpt;
use tonic::transport::Server;
use utils::communication::consumer::CommonConsumer;
use utils::{metrics, status_endpoints};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let config = RegistryConfig::from_args();

    env_logger::init();

    debug!("Environment: {:?}", config);

    status_endpoints::serve();
    metrics::serve(config.metrics_port);

    let registry = EdgeRegistryImpl::new(&config).await?;

    let consumer = CommonConsumer::new((&config.consumer_config).into()).await?;

    let handler = registry.clone();
    tokio::spawn(async {
        info!("Listening for messages via MQ");
        match consumer.run(handler).await {
            Ok(_) => {
                error!("MQ consumer finished work"); // If this happens it means that there's problem with Kafka or AMQP connection
            }
            Err(err) => {
                error!("MQ consumer returned with error: {:?}", err);
            }
        }
        process::abort();
    });

    status_endpoints::mark_as_started();
    info!("Starting a grpc server");
    Server::builder()
        .add_service(EdgeRegistryServer::new(registry))
        .serve(([0, 0, 0, 0], config.communication_port).into())
        .await?;

    Ok(())
}

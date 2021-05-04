use clap::Clap;
use edge_registry::args::{ConsumerMethod, RegistryConfig};
use edge_registry::EdgeRegistryImpl;
use rpc::edge_registry::edge_registry_server::EdgeRegistryServer;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tracing::{debug, error, info};
use utils::communication::consumer::CommonConsumer;
use utils::communication::publisher::CommonPublisher;
use utils::notification::full_notification_sender::FullNotificationSenderBase;
use utils::notification::NotificationSender;
use utils::{metrics, status_endpoints};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let config: RegistryConfig = RegistryConfig::parse();

    debug!("Environment: {:?}", config);

    status_endpoints::serve(config.status_port);
    metrics::serve(config.metrics_port);

    let notification_sender = match &config.notification_config.destination {
        Some(destination) => {
            let publisher = match config.consumer_config.consumer_method {
                ConsumerMethod::Kafka => {
                    CommonPublisher::new_kafka(&config.consumer_config.consumer_host).await?
                }
                ConsumerMethod::Amqp => {
                    CommonPublisher::new_amqp(&config.consumer_config.consumer_host).await?
                }
            };

            NotificationSender::Full(
                FullNotificationSenderBase::new(
                    publisher,
                    destination.clone(),
                    "Base".to_string(),
                    "EdgeRegistry",
                )
                .await,
            )
        }
        None => NotificationSender::Disabled,
    };

    let notification_sender = Arc::new(Mutex::new(notification_sender));

    let registry = EdgeRegistryImpl::new(&config, notification_sender).await?;

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
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        process::abort();
    });

    status_endpoints::mark_as_started();
    info!("Starting a grpc server");
    Server::builder()
        .trace_fn(utils::tracing::grpc::trace_fn)
        .add_service(EdgeRegistryServer::new(registry))
        .serve(([0, 0, 0, 0], config.rpc_port).into())
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

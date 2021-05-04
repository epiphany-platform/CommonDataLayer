#![feature(async_closure)]

use edge_registry::settings::Settings;
use edge_registry::EdgeRegistryImpl;
use rpc::edge_registry::edge_registry_server::EdgeRegistryServer;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tracing::{error, info};
use utils::settings::{load_settings, CommunicationMethod};
use utils::{metrics, status_endpoints};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    settings.log.init()?;

    tracing::debug!(?settings, "command-line arguments");

    status_endpoints::serve(&settings.monitoring);
    metrics::serve(&settings.monitoring);

    let notification_publisher = settings
        .notifications
        .publisher(
            settings.publisher().await?,
            settings.communication_method.to_string(),
            "EdgeRegistry",
        )
        .await;

    let registry = EdgeRegistryImpl::new(
        &settings.postgres,
        Arc::new(Mutex::new(notification_publisher)),
    )
    .await?;
    let consumer = match (settings.kafka, settings.amqp) {
        (Some(kafka), _) if settings.communication_method == CommunicationMethod::Kafka => {
            Some(kafka.consumer().await?)
        }
        (_, Some(amqp)) if settings.communication_method == CommunicationMethod::Amqp => {
            Some(amqp.consumer().await?)
        }
        _ => None, // Supported by default, we can skip
    };

    if let Some(consumer) = consumer {
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
    }

    status_endpoints::mark_as_started();
    info!("Starting a grpc server");
    Server::builder()
        .add_service(EdgeRegistryServer::new(registry))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

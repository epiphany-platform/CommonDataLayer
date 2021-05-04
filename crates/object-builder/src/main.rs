use object_builder::{settings::Settings, ObjectBuilderImpl};
use rpc::object_builder::object_builder_server::ObjectBuilderServer;
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

    let object_builder = ObjectBuilderImpl::new(&settings.services.schema_registry_url).await?;
    let consumer = settings.consumer().await?;
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

    utils::status_endpoints::mark_as_started();

    Server::builder()
        .add_service(ObjectBuilderServer::new(object_builder))
        .serve(([0, 0, 0, 0], settings.input_port).into())
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

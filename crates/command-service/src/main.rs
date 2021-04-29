use anyhow::bail;
use command_service::communication::MessageRouter;
use command_service::input::{Error, Service};
use command_service::output::{
    DruidOutputPlugin, OutputPlugin, PostgresOutputPlugin, VictoriaMetricsOutputPlugin,
};
use command_service::settings::{RepositoryKind, Settings};
use tracing::debug;
use utils::communication::parallel_consumer::ParallelCommonConsumer;
use utils::message_types::OwnedInsertMessage;
use utils::metrics;
use utils::notification::NotificationPublisher;
use utils::settings::load_settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let settings: Settings = load_settings()?;

    debug!("Environment: {:?}", settings);

    metrics::serve(&settings.monitoring);

    let consumers = settings.consumers(settings.async_task_limit).await?;
    let notification_publisher = settings
        .notifications
        .publisher(
            settings.publisher().await?,
            settings.communication_method.to_string(),
            "CommandService",
        )
        .await;

    match (settings.postgres, settings.victoria_metrics, settings.druid) {
        (Some(postgres), _, _) if settings.repository_kind == RepositoryKind::Postgres => {
            start_services(
                consumers,
                notification_publisher,
                PostgresOutputPlugin::new(postgres).await?,
            )
            .await
        }
        (_, Some(victoria_metrics), _)
            if settings.repository_kind == RepositoryKind::VictoriaMetrics =>
        {
            start_services(
                consumers,
                notification_publisher,
                VictoriaMetricsOutputPlugin::new(victoria_metrics)?,
            )
            .await
        }
        (_, _, Some(druid)) if settings.repository_kind == RepositoryKind::Druid => {
            if let Some(kafka) = settings.kafka {
                start_services(
                    consumers,
                    notification_publisher,
                    DruidOutputPlugin::new(druid, &kafka.brokers).await?,
                )
                .await
            } else {
                bail!("Druid setup requires [kafka] section")
            }
        }
        _ => todo!(),
    }?;

    Ok(())
}

async fn start_services(
    communication_config: Vec<ParallelCommonConsumer>,
    notification_publisher: NotificationPublisher<OwnedInsertMessage>,
    output: impl OutputPlugin,
) -> Result<(), Error> {
    let message_router = MessageRouter::new(notification_publisher, output);

    debug!("Starting command service on a message-queue");
    Service::new(communication_config, message_router)
        .await?
        .listen()
        .await?;

    Ok(())
}

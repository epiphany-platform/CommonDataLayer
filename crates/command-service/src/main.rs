#![feature(async_closure)]

use anyhow::bail;
use cdl_dto::ingestion::OwnedInsertMessage;
use command_service::communication::MessageRouter;
use command_service::input::{Error, Service};
use command_service::output::{
    DruidOutputPlugin, OutputPlugin, PostgresOutputPlugin, VictoriaMetricsOutputPlugin,
};
use command_service::settings::{RepositoryKind, Settings};
use communication_utils::parallel_consumer::ParallelCommonConsumer;
use metrics_utils as metrics;
use settings_utils::load_settings;
use tracing::debug;
use utils::notification::NotificationPublisher;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    misc_utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    tracing_utils::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");

    metrics::serve(&settings.monitoring);

    let consumers = settings.consumers(settings.async_task_limit).await?;
    let notification_publisher = settings
        .notifications
        .publisher(
            async || settings.publisher().await,
            settings.communication_method.to_string(),
            "CommandService",
        )
        .await?;

    match (
        settings.postgres,
        settings.victoria_metrics,
        settings.druid,
        settings.repository_kind,
    ) {
        (Some(postgres), _, _, RepositoryKind::Postgres) => {
            start_services(
                consumers,
                notification_publisher,
                PostgresOutputPlugin::new(postgres).await?,
            )
            .await
        }
        (_, Some(victoria_metrics), _, RepositoryKind::VictoriaMetrics) => {
            start_services(
                consumers,
                notification_publisher,
                VictoriaMetricsOutputPlugin::new(victoria_metrics)?,
            )
            .await
        }
        (_, _, Some(druid), RepositoryKind::Druid) => {
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
        _ => bail!("Unsupported consumer specification"),
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

use clap::Clap;
use command_service::communication::MessageRouter;
use command_service::input::{Error, Service};
use command_service::output::{
    DruidOutputPlugin, OutputArgs, OutputPlugin, PostgresOutputPlugin, VictoriaMetricsOutputPlugin,
};
use command_service::{args::Args, communication::config::CommunicationConfig};
use tracing::debug;
use utils::communication::publisher::CommonPublisher;
use utils::metrics;
use utils::report::{FullReportSenderBase, ReportSender, ReportServiceConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();
    utils::tracing::init();

    let args: Args = Args::parse();

    debug!("Environment: {:?}", args);

    metrics::serve(args.metrics_port);

    let communication_config = args.communication_config()?;

    match args.output_config {
        OutputArgs::Postgres(postgres_config) => {
            start_services(
                communication_config,
                args.report_config,
                PostgresOutputPlugin::new(postgres_config).await?,
            )
            .await
        }
        OutputArgs::Druid(druid_config) => {
            start_services(
                communication_config,
                args.report_config,
                DruidOutputPlugin::new(druid_config).await?,
            )
            .await
        }
        OutputArgs::VictoriaMetrics(victoria_metrics_config) => {
            start_services(
                communication_config,
                args.report_config,
                VictoriaMetricsOutputPlugin::new(victoria_metrics_config)?,
            )
            .await
        }
    }?;

    Ok(())
}

async fn start_services(
    communication_config: CommunicationConfig,
    report_config: ReportServiceConfig,
    output: impl OutputPlugin,
) -> Result<(), Error> {
    let report_service = match report_config.destination {
        Some(destination) => {
            let publisher = match &communication_config {
                CommunicationConfig::Kafka { brokers, .. } => CommonPublisher::new_kafka(&brokers)
                    .await
                    .map_err(Error::ConsumerCreationFailed)?,
                CommunicationConfig::Amqp {
                    connection_string, ..
                } => CommonPublisher::new_amqp(&connection_string)
                    .await
                    .map_err(Error::ConsumerCreationFailed)?,
                CommunicationConfig::Grpc {
                    report_endpoint_url,
                    ..
                } => CommonPublisher::new_rest(report_endpoint_url.clone())
                    .await
                    .map_err(Error::ConsumerCreationFailed)?,
            };

            ReportSender::Full(
                FullReportSenderBase::new(publisher, destination, output.name().to_string())
                    .await
                    .map_err(Error::FailedToInitializeReporting)?,
            )
        }
        None => ReportSender::Disabled,
    };

    let message_router = MessageRouter::new(report_service, output);

    debug!("Starting command service on a message-queue");
    Service::new(communication_config, message_router)
        .await?
        .listen()
        .await?;

    Ok(())
}

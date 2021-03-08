use command_service::communication::MessageRouter;
use command_service::input::{Error, Service};
use command_service::output::{
    DruidOutputPlugin, OutputArgs, OutputPlugin, PostgresOutputPlugin, VictoriaMetricsOutputPlugin,
};
use command_service::report::{FullReportSenderBase, ReportSender, ReportServiceConfig};
use command_service::{args::Args, communication::config::CommunicationConfig};
use log::debug;
use structopt::StructOpt;
use utils::metrics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args: Args = Args::from_args();

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
        Some(destination) => ReportSender::Full(
            FullReportSenderBase::new(
                &communication_config,
                destination,
                output.name().to_string(),
            )
            .await
            .map_err(Error::FailedToInitializeReporting)?,
        ),
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

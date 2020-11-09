use command_service::args::Args;
use command_service::communication::MessageRouter;
use command_service::input::{Error, KafkaInput, KafkaInputConfig};
use command_service::output::{
    DruidOutputPlugin, OutputArgs, OutputPlugin, PostgresOutputPlugin, SleighOutputPlugin,
    VictoriaMetricsOutputPlugin,
};
use command_service::report::{FullReportServiceConfig, ReportService, ReportServiceConfig};
use log::trace;
use structopt::StructOpt;
use utils::metrics;

#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Args = Args::from_args();

    trace!("Environment: {:?}", args);

    metrics::serve();
    match args.output_config {
        OutputArgs::Sleigh(sleigh_config) => {
            start_services(
                args.input_config,
                args.report_config,
                SleighOutputPlugin::new(sleigh_config)
                    .await
                    .expect("Initializing Sleigh output interface"),
            )
            .await
        }
        OutputArgs::Postgres(postgres_config) => {
            start_services(
                args.input_config,
                args.report_config,
                PostgresOutputPlugin::new(postgres_config)
                    .await
                    .expect("Initializing Postgres output interface"),
            )
            .await
        }
        OutputArgs::Druid(druid_config) => {
            start_services(
                args.input_config,
                args.report_config,
                DruidOutputPlugin::new(druid_config)
                    .await
                    .expect("Initializing Druid output interface"),
            )
            .await
        }
        OutputArgs::VictoriaMetrics(victoria_metrics_config) => {
            start_services(
                args.input_config,
                args.report_config,
                VictoriaMetricsOutputPlugin::new(victoria_metrics_config)
                    .expect("Initializing Victoria Metrics output interface"),
            )
            .await
        }
    }
    .expect("Running services")
}

async fn start_services(
    input_config: KafkaInputConfig,
    report_config: ReportServiceConfig,
    output: impl OutputPlugin,
) -> Result<(), Error> {
    let report_service = match (report_config.topic, report_config.broker) {
        (Some(topic), Some(broker)) => ReportService::Full(
            FullReportServiceConfig::new(broker, topic, output.name().to_string())
                .map_err(Error::FailedToInitializeReporting)?,
        ),
        (None, None) => ReportService::Disabled,
        _ => panic!("Must provide both topic and brokers for reporting service to enable it"),
    };

    let message_router = MessageRouter::new(report_service, output);

    let input_service = KafkaInput::new(input_config, message_router)?;

    input_service.listen().await
}

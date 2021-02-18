use command_service::communication::MessageRouter;
use command_service::input::{Error, MessageQueueInput};
use command_service::output::{
    DruidOutputPlugin, OutputArgs, OutputPlugin, PostgresOutputPlugin, VictoriaMetricsOutputPlugin,
};
use command_service::report::{FullReportSenderBase, ReportSender, ReportServiceConfig};
use command_service::{args::Args, communication::config::CommunicationConfig, input::GRPCInput};
use log::debug;
use rpc::command_service::command_service_server::CommandServiceServer;
use std::net::{Ipv4Addr, SocketAddrV4};
use structopt::StructOpt;
use tonic::transport::Server;
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
    let report_service = match report_config.topic_or_exchange {
        Some(topic_or_exchange) => ReportSender::Full(
            FullReportSenderBase::new(
                &communication_config,
                topic_or_exchange,
                output.name().to_string(),
            )
            .await
            .map_err(Error::FailedToInitializeReporting)?,
        ),
        None => ReportSender::Disabled,
    };

    let message_router = MessageRouter::new(report_service, output);

    match communication_config {
        CommunicationConfig::MessageQueue(communication_config) => {
            debug!("Starting command service on a message-queue");
            MessageQueueInput::new(communication_config, message_router)
                .await?
                .listen()
                .await?
        }
        CommunicationConfig::GRpc(communication_config) => {
            debug!("Starting command service on a grpc");
            let input = GRPCInput::new(message_router);
            let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), communication_config.grpc_port);
            Server::builder()
                .add_service(CommandServiceServer::new(input))
                .serve(addr.into())
                .await?;
        }
    }

    Ok(())
}

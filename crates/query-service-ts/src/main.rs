use anyhow::Context;
use query_service_ts::druid::{DruidQuery, DruidSettings};
use query_service_ts::victoria::VictoriaQuery;
use rpc::query_service_ts::query_service_ts_server::{QueryServiceTs, QueryServiceTsServer};
use serde::Deserialize;
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;
use utils::metrics;
use utils::settings::*;

#[derive(Debug, Deserialize)]
pub struct Settings {
    repository_kind: RepositoryKind,
    input_port: u16,

    druid: Option<DruidSettings>,
    victoria_metrics: Option<VictoriaMetricsSettings>,

    monitoring: MonitoringSettings,

    #[serde(default)]
    log: LogSettings,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryKind {
    VictoriaMetrics,
    Druid,
}

//Could be extracted to utils, dunno how without schema
async fn spawn_server<Q: QueryServiceTs>(service: Q, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(utils::tracing::grpc::trace_fn)
        .add_service(QueryServiceTsServer::new(service))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::set_aborting_panic_hook();

    let settings: Settings = load_settings()?;
    ::utils::tracing::init(
        settings.log.rust_log.as_str(),
        settings.monitoring.otel_service_name.as_str(),
    )?;

    tracing::debug!(?settings, "application environment");
    metrics::serve(&settings.monitoring);

    match settings.repository_kind {
        RepositoryKind::VictoriaMetrics => {
            spawn_server(
                VictoriaQuery::load(settings.victoria_metrics.expect("victoria_metrics config"))
                    .await?,
                settings.input_port,
            )
            .await
        }
        RepositoryKind::Druid => {
            spawn_server(
                DruidQuery::load(settings.druid.expect("druid config")).await?,
                settings.input_port,
            )
            .await
        }
    }
}

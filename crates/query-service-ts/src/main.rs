use anyhow::Context;
use metrics_utils as metrics;
use query_service_ts::druid::{DruidQuery, DruidSettings};
use query_service_ts::victoria::VictoriaQuery;
use rpc::query_service_ts::query_service_ts_raw_server::{
    QueryServiceTsRaw, QueryServiceTsRawServer,
};
use rpc::query_service_ts::query_service_ts_server::{QueryServiceTs, QueryServiceTsServer};
use serde::Deserialize;
use settings_utils::*;
use std::net::{Ipv4Addr, SocketAddrV4};
use tonic::transport::Server;

#[derive(Debug, Deserialize)]
pub struct Settings {
    repository_kind: RepositoryKind,
    input_port: u16,

    druid: Option<DruidSettings>,
    victoria_metrics: Option<VictoriaMetricsSettings>,

    monitoring: MonitoringSettings,

    #[serde(default)]
    log: LogSettings,

    #[serde(default)]
    features: FeatureSettings,
}

#[derive(Debug, Deserialize)]
pub struct FeatureSettings {
    pub raw_endpoint: bool,
}

impl Default for FeatureSettings {
    fn default() -> Self {
        Self { raw_endpoint: true }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryKind {
    VictoriaMetrics,
    Druid,
}

//Could be extracted to utils, dunno how without schema
async fn spawn_server<Q: QueryServiceTs, R: QueryServiceTsRaw>(
    raw_service: Option<R>,
    main_service: Q,
    port: u16,
) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

    Server::builder()
        .trace_fn(tracing_utils::grpc::trace_fn)
        .add_service(QueryServiceTsServer::new(main_service))
        .add_optional_service(raw_service.map(QueryServiceTsRawServer::new))
        .serve(addr.into())
        .await
        .context("gRPC server failed")
}

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

    match settings.repository_kind {
        RepositoryKind::VictoriaMetrics => {
            let vm =
                VictoriaQuery::load(settings.victoria_metrics.expect("victoria_metrics config"))
                    .await?;
            spawn_server(
                settings.features.raw_endpoint.then(|| vm.clone()),
                vm,
                settings.input_port,
            )
            .await
        }
        RepositoryKind::Druid => {
            let druid = DruidQuery::load(settings.druid.expect("druid config")).await?;
            spawn_server(
                settings.features.raw_endpoint.then(|| druid.clone()),
                druid,
                settings.input_port,
            )
            .await
        }
    }
}

use anyhow::Context;
use log::debug;
use metrics_runtime::Receiver;
use std::net::{Ipv4Addr, SocketAddrV4};

pub use metrics::{counter, gauge, timing, value};

pub const DEFAULT_PORT: &str = "58105";

pub fn serve(port: u16) {
    tokio::spawn(setup_metrics(port));
}

async fn setup_metrics(port: u16) -> anyhow::Result<()> {
    let metrics_receiver = Receiver::builder()
        .build()
        .context("failed to create receiver")?;
    let controller = metrics_receiver.controller();
    metrics_receiver.install();

    let metrics_exporter = metrics_exporter_http::HttpExporter::new(
        controller,
        metrics_observer_prometheus::PrometheusBuilder::new(),
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port).into(),
    );

    debug!("Initializing metrics at port {}", port);

    metrics_exporter
        .async_run()
        .await
        .context("Failed to serve metrics")
}

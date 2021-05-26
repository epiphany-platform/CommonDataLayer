use std::net::{Ipv4Addr, SocketAddrV4};

pub use metrics::{self, counter, gauge, try_recorder, Key, SharedString};
use metrics_exporter_prometheus::PrometheusBuilder;
use settings_utils::MonitoringSettings;
use tracing::debug;

pub const DEFAULT_PORT: &str = "51805";

pub fn serve(settings: &MonitoringSettings) {
    debug!("Initializing metrics at port {}", settings.metrics_port);

    PrometheusBuilder::new()
        .listen_address(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            settings.metrics_port,
        ))
        .install()
        .expect("failed to install Prometheus recorder");
}

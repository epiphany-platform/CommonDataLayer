use std::net::{Ipv4Addr, SocketAddrV4};

use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::debug;

pub use metrics::{self, counter, gauge, try_recorder, Key, KeyData, SharedString};

pub const DEFAULT_PORT: &str = "51805";

pub fn serve(port: u16) {
    debug!("Initializing metrics at port {}", port);

    PrometheusBuilder::new()
        .listen_address(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port))
        .install()
        .expect("failed to install Prometheus recorder");
}

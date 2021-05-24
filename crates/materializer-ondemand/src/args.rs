use clap::Clap;

#[derive(Clap, Debug)]
pub struct Args {
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,
    /// Port to listen on for Prometheus requests
    #[clap(long, default_value = metrics_utils::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[clap(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,
    /// Object builder's address
    #[clap(long, env)]
    pub object_builder_addr: String,
}

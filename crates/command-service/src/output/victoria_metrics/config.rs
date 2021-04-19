use clap::Clap;
use reqwest::Url;

#[derive(Clone, Debug, Clap)]
pub struct VictoriaMetricsConfig {
    /// Address of Victoria Metrics
    #[clap(
        name = "victoria-metrics-url",
        long = "victoria-metrics-output-url",
        env = "VICTORIA_METRICS_OUTPUT_URL"
    )]
    pub url: Url,
}

use crate::communication::resolution::Resolution;
pub use crate::output::druid::{DruidOutputConfig, DruidOutputPlugin};
pub use crate::output::victoria_metrics::config::VictoriaMetricsConfig;
pub use crate::output::victoria_metrics::VictoriaMetricsOutputPlugin;
use clap::Clap;
pub use psql::{PostgresOutputConfig, PostgresOutputPlugin};
use utils::message_types::BorrowedInsertMessage;

mod druid;
mod psql;
mod victoria_metrics;

#[derive(Clone, Debug, Clap)]
pub enum OutputArgs {
    Postgres(PostgresOutputConfig),
    Druid(DruidOutputConfig),
    VictoriaMetrics(VictoriaMetricsConfig),
}

#[async_trait::async_trait]
pub trait OutputPlugin: Send + Sync + 'static {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution;
    fn name(&self) -> &'static str;
}

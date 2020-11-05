use structopt::StructOpt;

pub use error::OutputError;
pub use psql::{PostgresOutputConfig, PostgresOutputPlugin};
pub use sleigh::{SleighOutputConfig, SleighOutputPlugin};

use crate::communication::resolution::Resolution;
use crate::communication::GenericMessage;
pub use crate::output::druid::{DruidOutputConfig, DruidOutputPlugin};
pub use crate::output::victoria_metrics::config::VictoriaMetricsConfig;
pub use crate::output::victoria_metrics::VictoriaMetricsOutputPlugin;

mod druid;
mod error;
mod psql;
mod sleigh;
mod victoria_metrics;

#[derive(Clone, Debug, StructOpt)]
pub enum OutputArgs {
    Sleigh(SleighOutputConfig),
    Postgres(PostgresOutputConfig),
    Druid(DruidOutputConfig),
    VictoriaMetrics(VictoriaMetricsConfig),
}

#[async_trait::async_trait]
pub trait OutputPlugin {
    async fn handle_message(&self, msg: GenericMessage) -> Result<Resolution, OutputError>;
    fn name(&self) -> &'static str;
}

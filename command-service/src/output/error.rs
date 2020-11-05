use crate::output::{druid, psql, sleigh, victoria_metrics};
use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum OutputError {
    #[error("No output plugin was specified during compilation")]
    MissingOutputPlugin,
    #[error("Sleigh plugin failed: `{0}`")]
    SleighError(sleigh::Error),
    #[error("Postgres plugin failed `{0}`")]
    PostgresError(psql::Error),
    #[error("Druid plugin failed `{0}`")]
    DruidError(druid::Error),
    #[error("Victoria Metrics plugin failed `{0}`")]
    VictoriaMetricsError(victoria_metrics::Error),
    #[error("Unable to serve metrics API `{0}`")]
    UnableToServeMetrics(warp::Error),
}

impl From<sleigh::Error> for OutputError {
    fn from(err: sleigh::Error) -> Self {
        OutputError::SleighError(err)
    }
}

impl From<psql::Error> for OutputError {
    fn from(err: psql::Error) -> Self {
        OutputError::PostgresError(err)
    }
}

impl From<druid::Error> for OutputError {
    fn from(err: druid::Error) -> Self {
        OutputError::DruidError(err)
    }
}

impl From<victoria_metrics::Error> for OutputError {
    fn from(err: victoria_metrics::Error) -> Self {
        OutputError::VictoriaMetricsError(err)
    }
}

use serde::Deserialize;
use utils::settings::{MonitoringSettings, PostgresSettings, LogSettings};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input_port: u16,

    pub postgres: PostgresSettings,

    pub monitoring: MonitoringSettings,

    pub log: LogSettings,
}

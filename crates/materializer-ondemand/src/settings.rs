use serde::Deserialize;
use settings_utils::{LogSettings, MonitoringSettings};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input_port: u16,

    pub services: ServicesSettings,

    pub monitoring: MonitoringSettings,

    pub log: LogSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServicesSettings {
    pub object_builder_url: String,
}

use serde::Deserialize;
use utils::settings::MonitoringSettings;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input_port: u16,

    pub services: ServicesSettings,

    pub monitoring: MonitoringSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServicesSettings {
    pub object_builder_url: String,
}

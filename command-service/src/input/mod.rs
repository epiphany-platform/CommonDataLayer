pub use crate::input::config::{GRpcConfig, InputConfig, KafkaConfig};
pub use crate::input::error::Error;
pub use crate::input::service::{GRPCInput, KafkaInput};

mod config;
mod error;
mod service;

use crate::error::ClientError;
use schema_registry_client::SchemaRegistryClient;
use tonic::transport::Channel;

pub use crate::codegen::schema_registry::*;

pub async fn connect(addr: String) -> Result<SchemaRegistryClient<Channel>, ClientError> {
    SchemaRegistryClient::connect(addr)
        .await
        .map_err(|err| ClientError::ConnectionError {
            service: "schema registry",
            source: err,
        })
}

pub mod types {
    use std::convert::TryFrom;

    use async_graphql::Enum;
    use serde::{Deserialize, Serialize};
    use tonic::Status;

    use super::schema_type;

    #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, sqlx::Type, Enum)]
    #[sqlx(type_name = "schema_type_enum", rename_all = "lowercase")]
    pub enum SchemaType {
        DocumentStorage,
        Timeseries,
    }

    impl From<schema_type::Type> for SchemaType {
        fn from(st: schema_type::Type) -> Self {
            match st {
                schema_type::Type::DocumentStorage => SchemaType::DocumentStorage,
                schema_type::Type::Timeseries => SchemaType::Timeseries,
            }
        }
    }

    impl From<SchemaType> for schema_type::Type {
        fn from(st: SchemaType) -> Self {
            match st {
                SchemaType::DocumentStorage => schema_type::Type::DocumentStorage,
                SchemaType::Timeseries => schema_type::Type::Timeseries,
            }
        }
    }

    impl std::fmt::Display for SchemaType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                SchemaType::DocumentStorage => "DocumentStorage",
                SchemaType::Timeseries => "Timeseries",
            })
        }
    }

    impl std::str::FromStr for SchemaType {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "DocumentStorage" => Ok(SchemaType::DocumentStorage),
                "Timeseries" => Ok(SchemaType::Timeseries),
                invalid => Err(anyhow::anyhow!("Invalid schema type: {}", invalid)),
            }
        }
    }

    impl TryFrom<i32> for SchemaType {
        type Error = Status;

        fn try_from(variant: i32) -> Result<Self, Self::Error> {
            match variant {
                0 => Ok(SchemaType::DocumentStorage),
                1 => Ok(SchemaType::Timeseries),
                _ => Err(Status::invalid_argument("Invalid Schema Type")),
            }
        }
    }

    impl From<SchemaType> for i32 {
        fn from(r#type: SchemaType) -> i32 {
            let r#type: schema_type::Type = r#type.into();
            r#type as i32
        }
    }
}

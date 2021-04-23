use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldDefinition {
    FieldName(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub struct Request {
    pub view_id: Uuid,
    pub schemas: HashMap<Uuid, Schema>,
}

impl Request {
    pub fn new(view_id: Uuid) -> Self {
        Self {
            view_id,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub struct Schema {
    pub object_ids: HashSet<Uuid>,
}

impl TryFrom<rpc::object_builder::View> for Request {
    type Error = uuid::Error;

    fn try_from(value: rpc::object_builder::View) -> Result<Self, Self::Error> {
        let view_id = value.view_id.parse()?;
        let schemas = value
            .schemas
            .into_iter()
            .map(|(schema_id, schema)| {
                let schema_id: Uuid = schema_id.parse()?;

                let object_ids = schema
                    .object_ids
                    .into_iter()
                    .map(|id| {
                        let id: Uuid = id.parse()?;
                        Ok(id)
                    })
                    .collect::<Result<_, Self::Error>>()?;

                Ok((schema_id, Schema { object_ids }))
            })
            .collect::<Result<_, Self::Error>>()?;

        Ok(Self { view_id, schemas })
    }
}

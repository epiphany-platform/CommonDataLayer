use cdl_dto::materialization::FieldType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::ObjectIdPair;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum FieldDefinitionSource {
    Simple {
        object: ObjectIdPair,
        field_name: String,
        field_type: FieldType,
    },
    Computed {
        computation: ComputationSource,
        field_type: FieldType,
    },
    Array {
        fields: HashMap<String, FieldDefinitionSource>,
    },
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub enum ComputationSource {
    RawValue {
        value: Value,
    },
    FieldValue {
        object: ObjectIdPair,
        field_path: String,
    },
    Equals {
        lhs: Box<ComputationSource>,
        rhs: Box<ComputationSource>,
    },
}

#[derive(Debug, PartialEq)]
pub enum RowSource {
    Join {
        objects: HashMap<ObjectIdPair, Value>,
        root_object: ObjectIdPair,
        fields: HashMap<String, FieldDefinitionSource>,
    },
    Single {
        root_object: ObjectIdPair,
        value: Value,
        fields: HashMap<String, FieldDefinitionSource>,
    },
}

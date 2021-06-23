use cdl_dto::materialization::FieldType;
use rpc::schema_registry::types::LogicOperator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::ObjectIdPair;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
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
#[serde(rename_all = "snake_case")]
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
        filters: Option<FilterSource>,
    },
    Single {
        root_object: ObjectIdPair,
        value: Value,
        fields: HashMap<String, FieldDefinitionSource>,
        filters: Option<FilterSource>,
    },
}

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterSource {
    Equals {
        lhs: FilterValueSource,
        rhs: FilterValueSource,
    },
    Complex {
        operator: LogicOperator,
        operands: Vec<FilterSource>,
    },
}

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterValueSource {
    SchemaField {
        object: ObjectIdPair,
        field_path: String,
    },
    ViewPath {
        field_path: String,
    },
    RawValue {
        value: Value,
    },
    Computed {
        computation: ComputationSource,
    },
}

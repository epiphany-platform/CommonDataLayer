// TODO: Refactor: Split me into smaller modules

use std::{
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    num::NonZeroU8,
};

use rpc::schema_registry::types::SearchFor;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type LocalId = u8; // ID from Relation->local_id. 0 for base_schema_id.

use async_graphql::{Json, SimpleObject, Union};
use rpc::schema_registry::types::LogicOperator;

use crate::{RequestError, RequestResult, ResponseResult, TryFromRpc, TryIntoRpc};

/// View's filter
#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[serde(rename_all = "snake_case")]
#[rpc(
    rpc = "rpc::schema_registry::Filter",
    tag = "filter_kind",
    tag_rpc = "rpc::schema_registry::filter::FilterKind"
)]
pub enum Filter {
    #[rpc(rename = "Simple")]
    SimpleFilter(SimpleFilter),
    #[rpc(rename = "Complex")]
    ComplexFilter(ComplexFilter),
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[rpc(transparent, rpc = "rpc::schema_registry::SimpleFilter")]
pub struct SimpleFilter {
    pub filter: SimpleFilterKind,
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[serde(rename_all = "snake_case")]
#[rpc(
    rpc = "rpc::schema_registry::SimpleFilter",
    tag = "simple_filter",
    tag_rpc = "rpc::schema_registry::simple_filter::SimpleFilter"
)]
pub enum SimpleFilterKind {
    Equals(EqualsFilter),
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[rpc(rpc = "rpc::schema_registry::EqualsFilter")]
pub struct EqualsFilter {
    pub lhs: FilterValue,
    pub rhs: FilterValue,
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[serde(rename_all = "snake_case")]
#[rpc(
    rpc = "rpc::schema_registry::FilterValue",
    tag = "filter_value",
    tag_rpc = "rpc::schema_registry::filter_value::FilterValue"
)]
pub enum FilterValue {
    SchemaField(SchemaFieldFilter),
    ViewPath(ViewPathFilter),
    RawValue(RawValueFilter),
    Computed(ComputedFilter),
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct SchemaFieldFilter {
    pub schema_id: LocalId,
    pub field_path: String,
}

impl TryFromRpc<rpc::schema_registry::SchemaFieldFilter> for SchemaFieldFilter {
    fn try_from_rpc(rpc: rpc::schema_registry::SchemaFieldFilter) -> RequestResult<Self> {
        Ok(Self {
            schema_id: rpc.schema_id.try_into()?,
            field_path: rpc.field_path,
        })
    }
}

impl TryIntoRpc for SchemaFieldFilter {
    type Rpc = rpc::schema_registry::SchemaFieldFilter;
    fn try_into_rpc(self) -> ResponseResult<rpc::schema_registry::SchemaFieldFilter> {
        Ok(rpc::schema_registry::SchemaFieldFilter {
            schema_id: self.schema_id.into(),
            field_path: self.field_path,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ViewPathFilter {
    pub field_path: String,
}

impl TryFromRpc<rpc::schema_registry::ViewPathFilter> for ViewPathFilter {
    fn try_from_rpc(rpc: rpc::schema_registry::ViewPathFilter) -> RequestResult<Self> {
        Ok(Self {
            field_path: rpc.field_path,
        })
    }
}

impl TryIntoRpc for ViewPathFilter {
    type Rpc = rpc::schema_registry::ViewPathFilter;
    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::schema_registry::ViewPathFilter {
            field_path: self.field_path,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct RawValueFilter {
    pub value: Json<Value>,
}

impl TryFromRpc<rpc::schema_registry::RawValueFilter> for RawValueFilter {
    fn try_from_rpc(rpc: rpc::schema_registry::RawValueFilter) -> RequestResult<Self> {
        Ok(Self {
            value: Json(serde_json::from_str(&rpc.value)?),
        })
    }
}

impl TryIntoRpc for RawValueFilter {
    type Rpc = rpc::schema_registry::RawValueFilter;
    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::schema_registry::RawValueFilter {
            value: serde_json::to_string(&self.value.0)?,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[rpc(rpc = "rpc::schema_registry::ComputedFilter")]
pub struct ComputedFilter {
    pub computation: Computation,
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[serde(rename_all = "snake_case")]
#[rpc(
    rpc = "rpc::schema_registry::Computation",
    tag = "computation",
    tag_rpc = "rpc::schema_registry::computation::Computation"
)]
pub enum Computation {
    RawValue(RawValueComputation),
    FieldValue(FieldValueComputation),
    #[rpc(rename = "EqualsComputation")]
    #[rpc(into_boxed)]
    Equals(EqualsComputation),
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct RawValueComputation {
    pub value: Json<Value>,
}

impl TryFromRpc<rpc::schema_registry::RawValueComputation> for RawValueComputation {
    fn try_from_rpc(rpc: rpc::schema_registry::RawValueComputation) -> RequestResult<Self> {
        Ok(Self {
            value: Json(serde_json::from_str(&rpc.value)?),
        })
    }
}

impl TryIntoRpc for RawValueComputation {
    type Rpc = rpc::schema_registry::RawValueComputation;
    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::schema_registry::RawValueComputation {
            value: serde_json::to_string(&self.value.0)?,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct FieldValueComputation {
    pub schema_id: LocalId,
    pub field_path: String,
}

impl TryFromRpc<rpc::schema_registry::FieldValueComputation> for FieldValueComputation {
    fn try_from_rpc(rpc: rpc::schema_registry::FieldValueComputation) -> RequestResult<Self> {
        Ok(Self {
            schema_id: rpc.schema_id.unwrap_or_default().try_into()?,
            field_path: rpc.field_path,
        })
    }
}

impl TryIntoRpc for FieldValueComputation {
    type Rpc = rpc::schema_registry::FieldValueComputation;

    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::schema_registry::FieldValueComputation {
            schema_id: match self.schema_id {
                0 => None,
                x => Some(x.into()),
            },
            field_path: self.field_path,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq, TryFromRpc, TryIntoRpc)]
#[rpc(rpc = "rpc::schema_registry::EqualsComputation")]
pub struct EqualsComputation {
    #[rpc(boxed, into_boxed)] // TODO: Document whats the difference between boxed and into_boxed
    pub lhs: Box<Computation>,
    #[rpc(boxed, into_boxed)]
    pub rhs: Box<Computation>,
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ComplexFilter {
    pub operator: LogicOperator,
    pub operands: Vec<Filter>,
}

impl TryFromRpc<rpc::schema_registry::ComplexFilter> for ComplexFilter {
    fn try_from_rpc(rpc: rpc::schema_registry::ComplexFilter) -> RequestResult<Self> {
        Ok(Self {
            operator: rpc.operator.try_into()?,
            operands: rpc
                .operands
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<RequestResult<_>>()?,
        })
    }
}

impl TryIntoRpc for ComplexFilter {
    type Rpc = rpc::schema_registry::ComplexFilter;

    fn try_into_rpc(self) -> ResponseResult<Self::Rpc> {
        Ok(rpc::schema_registry::ComplexFilter {
            operator: self.operator.into(),
            operands: self
                .operands
                .into_iter()
                .map(|o| o.try_into_rpc())
                .collect::<ResponseResult<_>>()?,
        })
    }
}

/// Relation between a view's schemas
#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    /// Relation ID stored in Edge Registry
    pub global_id: Uuid,
    /// Unique in view definition
    pub local_id: NonZeroU8,
    /// Looking at relation which direction is important.
    pub search_for: SearchFor,
    /// Subrelations
    #[serde(default)]
    pub relations: Vec<Relation>,
}

impl TryFromRpc<rpc::schema_registry::Relation> for Relation {
    fn try_from_rpc(rpc: rpc::schema_registry::Relation) -> RequestResult<Self> {
        Ok(Self {
            global_id: Uuid::parse_str(&rpc.global_id)?,
            local_id: create_non_zero_u8(rpc.local_id)?,
            search_for: rpc.search_for.try_into()?,
            relations: rpc
                .relations
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<RequestResult<_>>()?,
        })
    }
}

impl Relation {
    pub fn into_rpc(self) -> rpc::schema_registry::Relation {
        let local_id: u8 = self.local_id.into();
        rpc::schema_registry::Relation {
            global_id: self.global_id.to_string(),
            local_id: local_id as u32,
            search_for: self.search_for.into(),
            relations: self.relations.into_iter().map(|r| r.into_rpc()).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldDefinition {
    Simple {
        field_name: String,
        field_type: FieldType,
    },
    Computed {
        computation: Computation,
        field_type: FieldType,
    },
    Array {
        base: LocalId,
        fields: HashMap<String, FieldDefinition>,
    },
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum FieldType {
    String,
    Numeric,
    Json,
}

impl<'de> Deserialize<'de> for FieldType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VariantVisitor;

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = FieldType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`string`, `numeric` or `json`")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let lower = v.to_lowercase();
                match lower.as_str() {
                    "string" => Ok(FieldType::String),
                    "numeric" => Ok(FieldType::Numeric),
                    "json" => Ok(FieldType::Json),
                    _ => Err(serde::de::Error::invalid_type(
                        serde::de::Unexpected::Str(v),
                        &v,
                    )),
                }
            }
        }

        deserializer.deserialize_str(VariantVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
// Todo: probably unify it with api::types::view::queried::FullView
pub struct FullView {
    pub id: Uuid,
    pub base_schema_id: Uuid,
    pub name: String,
    pub materializer_address: String,
    pub materializer_options: Value,
    pub fields: HashMap<String, FieldDefinition>,
    pub relations: Vec<Relation>,
    pub filters: Option<Filter>,
}

impl TryFromRpc<rpc::schema_registry::FullView> for FullView {
    fn try_from_rpc(rpc: rpc::schema_registry::FullView) -> RequestResult<Self> {
        Ok(Self {
            id: rpc.id.parse()?,
            base_schema_id: rpc.base_schema_id.parse()?,
            name: rpc.name,
            materializer_address: rpc.materializer_address,
            materializer_options: serde_json::from_str(&rpc.materializer_options)?,
            fields: rpc
                .fields
                .into_iter()
                .map(|(key, field)| Ok((key, serde_json::from_str(&field)?)))
                .collect::<RequestResult<HashMap<_, _>>>()?,
            relations: rpc
                .relations
                .into_iter()
                .map(TryFromRpc::try_from_rpc)
                .collect::<RequestResult<_>>()?,
            filters: rpc.filters.map(TryFromRpc::try_from_rpc).transpose()?,
        })
    }
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

fn create_non_zero_u8(num: u32) -> RequestResult<NonZeroU8> {
    let num: u8 = num.try_into().map_err(|err| {
        RequestError::new(format!(
            "Unable to convert `{}` to 8bits unsigned integer: {:?}",
            num, err
        ))
    })?;
    NonZeroU8::new(num)
        .ok_or_else(|| RequestError::new(format!("Identifier `{}` is `0` - forbidden", num)))
}

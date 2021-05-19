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
use rpc::schema_registry::types::{FilterOperator, LogicOperator};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct RequestError(String);

impl RequestError {
    fn new(error: impl Into<String>) -> Self {
        Self(error.into())
    }
}

impl From<anyhow::Error> for RequestError {
    fn from(err: anyhow::Error) -> Self {
        Self(format!("{:?}", err))
    }
}

impl From<uuid::Error> for RequestError {
    fn from(err: uuid::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<std::num::TryFromIntError> for RequestError {
    fn from(err: std::num::TryFromIntError) -> Self {
        Self(err.to_string())
    }
}

impl From<RequestError> for tonic::Status {
    fn from(err: RequestError) -> Self {
        tonic::Status::invalid_argument(err.0)
    }
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ResponseError(String);

impl From<serde_json::Error> for ResponseError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }
}

impl From<ResponseError> for tonic::Status {
    fn from(err: ResponseError) -> Self {
        tonic::Status::internal(err.0)
    }
}

type RequestResult<T> = std::result::Result<T, RequestError>;
type ResponseResult<T> = std::result::Result<T, ResponseError>;

/// View's filter
#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
pub enum Filter {
    SimpleFilter(SimpleFilter),
    ComplexFilter(ComplexFilter),
}

impl Filter {
    pub fn from_rpc(rpc: rpc::schema_registry::Filter) -> RequestResult<Self> {
        let kind = match rpc.filter_kind {
            Some(kind) => kind,
            None => return Err(RequestError::new("Expected filter, found none")),
        };
        use rpc::schema_registry::filter::FilterKind;
        Ok(match kind {
            FilterKind::Simple(filter) => Self::SimpleFilter(SimpleFilter::from_rpc(filter)?),
            FilterKind::Complex(filter) => Self::ComplexFilter(ComplexFilter::from_rpc(filter)?),
        })
    }

    pub fn into_rpc(self) -> ResponseResult<rpc::schema_registry::Filter> {
        use rpc::schema_registry::filter::FilterKind;

        Ok(rpc::schema_registry::Filter {
            filter_kind: Some(match self {
                Filter::SimpleFilter(filter) => FilterKind::Simple(filter.into_rpc()?),
                Filter::ComplexFilter(filter) => FilterKind::Complex(filter.into_rpc()?),
            }),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct SimpleFilter {
    pub operator: FilterOperator,
    pub lhs: FilterValue,
    pub rhs: Option<FilterValue>,
}

impl SimpleFilter {
    fn from_rpc(rpc: rpc::schema_registry::SimpleFilter) -> RequestResult<Self> {
        Ok(Self {
            operator: rpc.operator.try_into()?,
            lhs: FilterValue::from_rpc(rpc.lhs)?,
            rhs: rpc.rhs.map(FilterValue::from_rpc).transpose()?,
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::SimpleFilter> {
        Ok(rpc::schema_registry::SimpleFilter {
            operator: self.operator.into(),
            lhs: self.lhs.into_rpc()?,
            rhs: self.rhs.map(|rhs| rhs.into_rpc()).transpose()?,
        })
    }
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
pub enum FilterValue {
    SchemaField(SchemaFieldFilter),
    ViewPath(ViewPathFilter),
    RawValue(RawValueFilter),
    Computed(ComputedFilter),
}

impl FilterValue {
    fn from_rpc(rpc: rpc::schema_registry::FilterValue) -> RequestResult<Self> {
        let kind = match rpc.filter_value {
            Some(kind) => kind,
            None => return Err(RequestError::new("Expected filter value, found none")),
        };
        use rpc::schema_registry::filter_value::FilterValue;
        Ok(match kind {
            FilterValue::SchemaField(filter) => {
                Self::SchemaField(SchemaFieldFilter::from_rpc(filter)?)
            }
            FilterValue::ViewPath(filter) => Self::ViewPath(ViewPathFilter::from_rpc(filter)),
            FilterValue::RawValue(filter) => Self::RawValue(RawValueFilter::from_rpc(filter)?),
            FilterValue::Computed(filter) => Self::Computed(ComputedFilter::from_rpc(filter)?),
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::FilterValue> {
        use rpc::schema_registry::filter_value::FilterValue;
        Ok(rpc::schema_registry::FilterValue {
            filter_value: Some(match self {
                Self::SchemaField(filter) => FilterValue::SchemaField(filter.into_rpc()),
                Self::ViewPath(filter) => FilterValue::ViewPath(filter.into_rpc()),
                Self::RawValue(filter) => FilterValue::RawValue(filter.into_rpc()?),
                Self::Computed(filter) => FilterValue::Computed(filter.into_rpc()?),
            }),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct SchemaFieldFilter {
    schema_id: LocalId,
    field_path: String,
}

impl SchemaFieldFilter {
    fn from_rpc(rpc: rpc::schema_registry::SchemaFieldFilter) -> RequestResult<Self> {
        Ok(Self {
            schema_id: rpc.schema_id.try_into()?,
            field_path: rpc.field_path,
        })
    }

    fn into_rpc(self) -> rpc::schema_registry::SchemaFieldFilter {
        rpc::schema_registry::SchemaFieldFilter {
            schema_id: self.schema_id.into(),
            field_path: self.field_path,
        }
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ViewPathFilter {
    field_path: String,
}

impl ViewPathFilter {
    fn from_rpc(rpc: rpc::schema_registry::ViewPathFilter) -> Self {
        Self {
            field_path: rpc.field_path,
        }
    }

    fn into_rpc(self) -> rpc::schema_registry::ViewPathFilter {
        rpc::schema_registry::ViewPathFilter {
            field_path: self.field_path,
        }
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct RawValueFilter {
    value: Json<Value>,
}

impl RawValueFilter {
    fn from_rpc(rpc: rpc::schema_registry::RawValueFilter) -> RequestResult<Self> {
        Ok(Self {
            value: Json(serde_json::from_str(&rpc.value)?),
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::RawValueFilter> {
        Ok(rpc::schema_registry::RawValueFilter {
            value: serde_json::to_string(&self.value.0)?,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ComputedFilter {
    computation: Computation,
}

impl ComputedFilter {
    fn from_rpc(rpc: rpc::schema_registry::ComputedFilter) -> RequestResult<Self> {
        Ok(Self {
            computation: Computation::from_rpc(&rpc.computation)?,
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::ComputedFilter> {
        Ok(rpc::schema_registry::ComputedFilter {
            computation: self.computation.into_rpc()?,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct Computation {
    pub operator: ComputationOperator,
    pub lhs: Box<Computation>,
    pub rhs: Option<Box<Computation>>,
}

impl Computation {
    fn from_rpc(rpc: &rpc::schema_registry::Computation) -> RequestResult<Self> {
        Ok(Self {
            operator: ComputationOperator::from_rpc(rpc.operator.clone())?,
            lhs: Box::new(Computation::from_rpc(&rpc.lhs)?),
            rhs: rpc
                .rhs
                .as_ref()
                .map(|rhs| RequestResult::Ok(Box::new(Computation::from_rpc(rhs)?)))
                .transpose()?,
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::Computation> {
        Ok(rpc::schema_registry::Computation {
            operator: self.operator.into_rpc()?,
            lhs: Box::new(self.lhs.into_rpc()?),
            rhs: self
                .rhs
                .map(|rhs| ResponseResult::Ok(Box::new(rhs.into_rpc()?)))
                .transpose()?,
        })
    }
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
pub enum ComputationOperator {
    RawValue(RawValueComputation),
    FieldValue(FieldValueComputation),
    Equals(EqualsComputation),
}

impl ComputationOperator {
    fn from_rpc(rpc: rpc::schema_registry::ComputationOperator) -> RequestResult<Self> {
        let kind = match rpc.computation_operator {
            Some(kind) => kind,
            None => {
                return Err(RequestError::new(
                    "Expected computation operator, found none",
                ))
            }
        };
        use rpc::schema_registry::computation_operator::ComputationOperator;
        Ok(match kind {
            ComputationOperator::RawValue(filter) => {
                Self::RawValue(RawValueComputation::from_rpc(filter)?)
            }
            ComputationOperator::FieldValue(filter) => {
                Self::FieldValue(FieldValueComputation::from_rpc(filter)?)
            }
            ComputationOperator::Equals(filter) => {
                Self::Equals(EqualsComputation::from_rpc(filter)?)
            }
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::ComputationOperator> {
        use rpc::schema_registry::computation_operator::ComputationOperator;
        Ok(rpc::schema_registry::ComputationOperator {
            computation_operator: Some(match self {
                Self::RawValue(op) => ComputationOperator::RawValue(op.into_rpc()?),
                Self::FieldValue(op) => ComputationOperator::FieldValue(op.into_rpc()),
                Self::Equals(op) => ComputationOperator::Equals(op.into_rpc()),
            }),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct RawValueComputation {
    value: Json<Value>,
}

impl RawValueComputation {
    fn from_rpc(rpc: rpc::schema_registry::RawValueComputation) -> RequestResult<Self> {
        Ok(Self {
            value: Json(serde_json::from_str(&rpc.value)?),
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::RawValueComputation> {
        Ok(rpc::schema_registry::RawValueComputation {
            value: serde_json::to_string(&self.value.0)?,
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct FieldValueComputation {
    base_schema: Option<Uuid>,
    field_path: String,
}

impl FieldValueComputation {
    fn from_rpc(rpc: rpc::schema_registry::FieldValueComputation) -> RequestResult<Self> {
        Ok(Self {
            base_schema: rpc.base_schema.map(|bs| bs.parse()).transpose()?,
            field_path: rpc.field_path,
        })
    }

    fn into_rpc(self) -> rpc::schema_registry::FieldValueComputation {
        rpc::schema_registry::FieldValueComputation {
            base_schema: self.base_schema.map(|s| s.to_string()),
            field_path: self.field_path,
        }
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct EqualsComputation {
    _placeholder: bool, // Hack for "SimpleObject needs at least one field"
}

impl EqualsComputation {
    fn from_rpc(_rpc: rpc::schema_registry::EqualsComputation) -> RequestResult<Self> {
        Ok(EqualsComputation { _placeholder: true })
    }

    fn into_rpc(self) -> rpc::schema_registry::EqualsComputation {
        rpc::schema_registry::EqualsComputation {}
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ComplexFilter {
    operator: LogicOperator,
    operands: Vec<Filter>,
}

impl ComplexFilter {
    fn from_rpc(rpc: rpc::schema_registry::ComplexFilter) -> RequestResult<Self> {
        Ok(Self {
            operator: rpc.operator.try_into()?,
            operands: rpc
                .operands
                .into_iter()
                .map(Filter::from_rpc)
                .collect::<RequestResult<_>>()?,
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::ComplexFilter> {
        Ok(rpc::schema_registry::ComplexFilter {
            operator: self.operator.into(),
            operands: self
                .operands
                .into_iter()
                .map(|o| o.into_rpc())
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

impl Relation {
    pub fn from_rpc(relation: rpc::schema_registry::Relation) -> RequestResult<Self> {
        Ok(Self {
            global_id: Uuid::parse_str(&relation.global_id)?,
            local_id: create_non_zero_u8(relation.local_id)?,
            search_for: relation.search_for.try_into()?,
            relations: relation
                .relations
                .into_iter()
                .map(Relation::from_rpc)
                .collect::<RequestResult<_>>()?,
        })
    }

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

//fn parse_uuid(id: &str) -> Result<Uuid, tonic::Status> {
//Uuid::parse_str(id)
//.map_err(|err| tonic::Status::invalid_argument(format!("Failed to parse UUID: {}", err)))
//}

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

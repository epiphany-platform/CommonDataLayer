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

use crate::{RequestError, RequestResult, ResponseResult};

/// View's filter
#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
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
    pub filter: SimpleFilterKind,
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SimpleFilterKind {
    Equals(EqualsFilter),
}

impl SimpleFilter {
    fn from_rpc(rpc: rpc::schema_registry::SimpleFilter) -> RequestResult<Self> {
        let kind = match rpc.simple_filter {
            Some(kind) => kind,
            None => return Err(RequestError::new("Expected filter, found none")),
        };
        use rpc::schema_registry::simple_filter::SimpleFilter;
        use rpc::schema_registry::EqualsFilter as EqualsFilterRpc;
        Ok(Self {
            filter: match kind {
                SimpleFilter::Equals(EqualsFilterRpc { lhs, rhs }) => {
                    SimpleFilterKind::Equals(EqualsFilter {
                        lhs: FilterValue::from_rpc(lhs)?,
                        rhs: FilterValue::from_rpc(rhs)?,
                    })
                }
            },
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::SimpleFilter> {
        use rpc::schema_registry::simple_filter::SimpleFilter;
        use rpc::schema_registry::EqualsFilter as EqualsFilterRpc;
        Ok(rpc::schema_registry::SimpleFilter {
            simple_filter: Some(match self.filter {
                SimpleFilterKind::Equals(EqualsFilter { lhs, rhs }) => {
                    SimpleFilter::Equals(EqualsFilterRpc {
                        lhs: lhs.into_rpc()?,
                        rhs: rhs.into_rpc()?,
                    })
                }
            }),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct EqualsFilter {
    pub lhs: FilterValue,
    pub rhs: FilterValue,
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
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
    pub schema_id: LocalId,
    pub field_path: String,
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
    pub field_path: String,
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
#[serde(transparent)]
pub struct RawValueFilter {
    pub value: Json<Value>,
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
    pub computation: Computation,
}

impl ComputedFilter {
    fn from_rpc(rpc: rpc::schema_registry::ComputedFilter) -> RequestResult<Self> {
        Ok(Self {
            computation: Computation::from_rpc(rpc.computation)?,
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::ComputedFilter> {
        Ok(rpc::schema_registry::ComputedFilter {
            computation: self.computation.into_rpc()?,
        })
    }
}

#[derive(Clone, Debug, Union, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Computation {
    RawValue(RawValueComputation),
    FieldValue(FieldValueComputation),
    Equals(EqualsComputation),
}

impl Computation {
    fn from_rpc(rpc: rpc::schema_registry::Computation) -> RequestResult<Self> {
        let kind = match rpc.computation {
            Some(kind) => kind,
            None => return Err(RequestError::new("Expected computation, found none")),
        };
        use rpc::schema_registry::computation::Computation;
        Ok(match kind {
            Computation::RawValue(filter) => Self::RawValue(RawValueComputation::from_rpc(filter)?),
            Computation::FieldValue(filter) => {
                Self::FieldValue(FieldValueComputation::from_rpc(filter)?)
            }
            Computation::EqualsComputation(filter) => {
                Self::Equals(EqualsComputation::from_rpc(*filter)?)
            }
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::Computation> {
        use rpc::schema_registry::computation::Computation;
        Ok(rpc::schema_registry::Computation {
            computation: Some(match self {
                Self::RawValue(op) => Computation::RawValue(op.into_rpc()?),
                Self::FieldValue(op) => Computation::FieldValue(op.into_rpc()),
                Self::Equals(op) => Computation::EqualsComputation(Box::new(op.into_rpc()?)),
            }),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct RawValueComputation {
    pub value: Json<Value>,
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
    pub schema_id: LocalId,
    pub field_path: String,
}

impl FieldValueComputation {
    fn from_rpc(rpc: rpc::schema_registry::FieldValueComputation) -> RequestResult<Self> {
        Ok(Self {
            schema_id: rpc.schema_id.unwrap_or_default().try_into()?,
            field_path: rpc.field_path,
        })
    }

    fn into_rpc(self) -> rpc::schema_registry::FieldValueComputation {
        rpc::schema_registry::FieldValueComputation {
            schema_id: match self.schema_id {
                0 => None,
                x => Some(x.into()),
            },
            field_path: self.field_path,
        }
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct EqualsComputation {
    pub lhs: Box<Computation>,
    pub rhs: Box<Computation>,
}

impl EqualsComputation {
    fn from_rpc(rpc: rpc::schema_registry::EqualsComputation) -> RequestResult<Self> {
        Ok(EqualsComputation {
            lhs: Box::new(Computation::from_rpc(*rpc.lhs)?),
            rhs: Box::new(Computation::from_rpc(*rpc.rhs)?),
        })
    }

    fn into_rpc(self) -> ResponseResult<rpc::schema_registry::EqualsComputation> {
        Ok(rpc::schema_registry::EqualsComputation {
            lhs: Box::new(self.lhs.into_rpc()?),
            rhs: Box::new(self.rhs.into_rpc()?),
        })
    }
}

#[derive(Clone, Debug, SimpleObject, Serialize, Deserialize, PartialEq)]
pub struct ComplexFilter {
    pub operator: LogicOperator,
    pub operands: Vec<Filter>,
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

impl FullView {
    pub fn from_rpc(rpc: rpc::schema_registry::FullView) -> RequestResult<Self> {
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
                .map(Relation::from_rpc)
                .collect::<RequestResult<_>>()?,
            filters: rpc.filters.map(Filter::from_rpc).transpose()?,
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

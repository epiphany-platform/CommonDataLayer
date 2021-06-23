use std::collections::HashMap;

use anyhow::{Context, Result};
use rpc::schema_registry::types::LogicOperator;
use serde_json::Value;

use crate::{
    sources::{FilterSource, FilterValueSource},
    utils::get_sub_object,
    ObjectIdPair, RowDefinition,
};

use super::field_builder::ComputationEngine;

pub struct RowFilter<'a> {
    objects: &'a HashMap<ObjectIdPair, Value>,
}

impl<'a> RowFilter<'a> {
    pub fn new(objects: &'a HashMap<ObjectIdPair, Value>) -> Self {
        Self { objects }
    }

    // TODO: In the future split it into post_filter and pre_filter, because sometimes you might want to filter out row before building it.
    pub fn filter(
        &self,
        row: RowDefinition,
        filters: Option<FilterSource>,
    ) -> Result<Option<RowDefinition>> {
        Ok(match filters {
            None => Some(row),
            Some(filters) => {
                if self.filter_row(&row, &filters)? {
                    Some(row)
                } else {
                    None
                }
            }
        })
    }

    fn filter_row(&self, row: &RowDefinition, filter: &FilterSource) -> Result<bool> {
        Ok(match filter {
            FilterSource::Equals { lhs, rhs } => {
                let lhs = self.filter_value(row, lhs)?;
                let rhs = self.filter_value(row, rhs)?;
                lhs == rhs
            }
            FilterSource::Complex { operator, operands } => {
                let mut operands = operands
                    .iter()
                    .map(|op| self.filter_row(row, op))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter();

                match operator {
                    LogicOperator::And => operands.all(|o| o),
                    LogicOperator::Or => operands.any(|o| o),
                }
            }
        })
    }

    fn filter_value(&self, row: &RowDefinition, value: &FilterValueSource) -> Result<Value> {
        Ok(match value {
            FilterValueSource::SchemaField { object, field_path } => {
                let object = self
                    .objects
                    .get(&object)
                    .with_context(|| format!("Could not find object: {:?}", object))?;
                let field_path_parts = field_path.split('.');
                get_sub_object(object, field_path_parts)?
            }
            FilterValueSource::ViewPath { field_path } => {
                let mut field_path_parts = field_path.split('.');
                let root = field_path_parts
                    .next()
                    .context("Expected at least one segment in field_path")?;
                let field = row
                    .fields
                    .get(root)
                    .with_context(|| format!("Could not find a row field named: `{}`", root))?;
                get_sub_object(field, field_path_parts)?
            }
            FilterValueSource::RawValue { value } => value.clone(),
            FilterValueSource::Computed { computation } => {
                ComputationEngine::new(self.objects).compute(computation)?
            }
        })
    }
}

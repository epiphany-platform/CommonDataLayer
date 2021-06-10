use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::{
    row_builder::field_builder::ComputationEngine, FieldDefinitionSource, ObjectIdPair,
    RowDefinition, RowSource,
};

mod field_builder;

use field_builder::FieldBuilder;

pub struct RowBuilder {}

impl RowBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn build(&self, source: RowSource) -> Result<RowDefinition> {
        match source {
            RowSource::Join {
                objects, fields, ..
            } => self.build_join(objects, fields),
            RowSource::Single {
                root_object,
                value,
                fields,
            } => self.build_single(root_object, value, fields),
        }
    }

    fn build_join(
        &self,
        objects: HashMap<ObjectIdPair, Value>,
        fields: HashMap<String, FieldDefinitionSource>,
    ) -> Result<RowDefinition> {
        let field_builder = FieldBuilder { objects: &objects };

        let fields = fields
            .iter()
            .map(|field| field_builder.build(field))
            .collect::<anyhow::Result<_>>()?;
        let object_ids = objects
            .keys()
            .map(|object_pair| object_pair.object_id)
            .collect();
        Ok(RowDefinition { object_ids, fields })
    }

    fn build_single(
        &self,
        pair: ObjectIdPair,
        object_value: Value,
        fields: HashMap<String, FieldDefinitionSource>,
    ) -> Result<RowDefinition> {
        let object = object_value
            .as_object()
            .with_context(|| format!("Expected object ({}) to be a JSON object", pair.object_id))?;

        use FieldDefinitionSource::*;

        let fields = fields
            .iter()
            .map(|(field_def_key, field_def)| {
                Ok((
                    field_def_key.into(),
                    match field_def {
                        Simple { field_name, .. } => {
                            //TODO: Use field_type
                            let value = object.get(field_name).with_context(|| {
                                format!(
                                    "Object ({}) does not have a field named `{}`",
                                    pair.object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                        Computed { computation, .. } => ComputationEngine::Simple {
                            object_id: pair,
                            value: &object_value,
                        }
                        .compute(computation)?,
                        Array { .. } => {
                            anyhow::bail!(
                                "Array field definition is not supported in relation-less view"
                            )
                        }
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        let mut object_ids = HashSet::new();
        object_ids.insert(pair.object_id);
        Ok(RowDefinition { object_ids, fields })
    }
}

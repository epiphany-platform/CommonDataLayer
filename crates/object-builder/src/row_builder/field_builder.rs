use crate::models::ObjectIdPair;
use crate::sources::FieldDefinitionSource;
use anyhow::{Context, Result};
pub use computation::ComputationEngine;
use serde_json::Value;
use std::collections::HashMap;

mod computation;

#[derive(Clone, Copy)]
pub struct FieldBuilder<'a> {
    pub objects: &'a HashMap<ObjectIdPair, Value>,
}

impl<'a> FieldBuilder<'a> {
    pub fn build(
        self,
        (field_name, field_def): (&String, &FieldDefinitionSource),
    ) -> Result<(String, Value)> {
        use FieldDefinitionSource::*;

        Ok((
            field_name.into(),
            match field_def {
                Simple {
                    field_name, object, ..
                } => {
                    let object_value = self.objects.get(object).unwrap();
                    let object_value = object_value.as_object().with_context(|| {
                        format!("Expected object ({}) to be a JSON object", object.object_id)
                    })?;
                    let value = object_value.get(field_name).with_context(|| {
                        format!(
                            "Object ({}) does not have a field named `{}`",
                            object.object_id, field_name
                        )
                    })?;
                    value.clone()
                }
                Computed { computation, .. } => {
                    let engine: ComputationEngine = self.into();
                    engine.compute(computation)?
                }
                Array { fields } => {
                    let fields = fields
                        .iter()
                        .map(|field| self.build(field))
                        .collect::<anyhow::Result<_>>()?;
                    Value::Object(fields)
                }
            },
        ))
    }
}

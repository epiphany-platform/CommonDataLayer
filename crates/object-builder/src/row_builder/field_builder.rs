use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::{FieldDefinitionSource, ObjectIdPair};

mod computation;
pub use computation::ComputationEngine;

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
                Array { elements } => {
                    let elements = elements
                        .iter()
                        .map(|element| {
                            let fields = element
                                .fields
                                .iter()
                                .map(|field| self.build(field))
                                .collect::<anyhow::Result<_>>()?;
                            Ok(Value::Object(fields))
                        })
                        .collect::<anyhow::Result<_>>()?;
                    Value::Array(elements)
                }
            },
        ))
    }
}

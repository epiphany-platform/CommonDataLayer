use std::collections::HashMap;

use anyhow::{Context, Result};
use cdl_dto::{
    edges::TreeObject,
    materialization::{FieldDefinition, FullView},
};
use serde_json::Value;

use crate::{row_builder::utils::get_objects_ids_for_relation, ObjectIdPair};

mod computation;
use computation::ComputationEngine;

#[derive(Clone, Copy)]
pub struct FieldBuilder<'a> {
    pub object_pair: ObjectIdPair,
    pub objects: &'a HashMap<ObjectIdPair, Value>,
    pub tree_object: &'a TreeObject,
    pub view: &'a FullView,
}

impl<'a> FieldBuilder<'a> {
    pub fn with_object(mut self, object_pair: ObjectIdPair) -> Self {
        self.object_pair = object_pair;
        self
    }

    pub fn build(
        self,
        (field_name, field_def): (&String, &FieldDefinition),
    ) -> Result<(String, Value)> {
        use FieldDefinition::*;

        Ok((
            field_name.into(),
            match field_def {
                Simple { field_name, .. } => {
                    let object = self.objects.get(&self.object_pair).unwrap();
                    let object = object.as_object().with_context(|| {
                        format!(
                            "Expected object ({}) to be a JSON object",
                            self.object_pair.object_id
                        )
                    })?;
                    let value = object.get(field_name).with_context(|| {
                        format!(
                            "Object ({}) does not have a field named `{}`",
                            self.object_pair.object_id, field_name
                        )
                    })?;
                    value.clone()
                }
                Computed { computation, .. } => {
                    let engine: ComputationEngine = self.into();
                    engine.compute(computation)?
                }
                Array { base, fields } => {
                    let objects_ids =
                        get_objects_ids_for_relation(*base, self.view, self.tree_object)?;
                    let objects = objects_ids
                        .into_iter()
                        .map(|object_id| {
                            let field_builder = self.with_object(object_id);
                            let fields = fields
                                .iter()
                                .map(|field| field_builder.build(field))
                                .collect::<anyhow::Result<_>>()?;

                            Ok(Value::Object(fields))
                        })
                        .collect::<anyhow::Result<_>>()?;
                    Value::Array(objects)
                }
            },
        ))
    }
}

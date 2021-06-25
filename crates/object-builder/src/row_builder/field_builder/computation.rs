use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::{sources::ComputationSource, utils::get_sub_object, ObjectIdPair};

use super::FieldBuilder;

#[derive(Clone, Copy)]
pub struct ComputationEngine<'a> {
    objects: &'a HashMap<ObjectIdPair, Value>,
}

impl<'a> ComputationEngine<'a> {
    pub fn new(objects: &'a HashMap<ObjectIdPair, Value>) -> Self {
        Self { objects }
    }

    pub fn compute(self, computation: &ComputationSource) -> Result<Value> {
        Ok(match computation {
            ComputationSource::RawValue { value } => value.clone(),
            ComputationSource::FieldValue { object, field_path } => {
                let field_path_parts = field_path.split('.');
                let object = self
                    .objects
                    .get(&object)
                    .with_context(|| format!("Could not find object: {:?}", object))?;
                get_sub_object(object, field_path_parts)?
            }
            ComputationSource::Equals { lhs, rhs } => {
                let lhs = self.compute(lhs)?;
                let rhs = self.compute(rhs)?;
                Value::Bool(lhs == rhs)
            }
        })
    }
}

impl<'a> From<FieldBuilder<'a>> for ComputationEngine<'a> {
    fn from(fb: FieldBuilder<'a>) -> Self {
        let FieldBuilder { objects, .. } = fb;
        Self { objects }
    }
}

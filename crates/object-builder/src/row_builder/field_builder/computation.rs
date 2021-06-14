use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::{utils::get_sub_object, ComputationSource, ObjectIdPair};

use super::FieldBuilder;

#[derive(Clone, Copy)]
pub enum ComputationEngine<'a> {
    Join {
        objects: &'a HashMap<ObjectIdPair, Value>,
    },
    Simple {
        object_id: ObjectIdPair,
        value: &'a Value,
    },
}

impl<'a> ComputationEngine<'a> {
    pub fn compute(self, computation: &ComputationSource) -> Result<Value> {
        Ok(match computation {
            ComputationSource::RawValue { value } => value.clone(),
            ComputationSource::FieldValue { object, field_path } => match self {
                ComputationEngine::Join { objects } => {
                    let field_path_parts = field_path.split('.');
                    let object = objects.get(&object).unwrap();
                    get_sub_object(object, field_path_parts)?
                }
                ComputationEngine::Simple { .. } => {
                    anyhow::bail!("Field value computation is not allowed for relationless row")
                }
            },
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
        Self::Join { objects }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use misc_utils::serde_json;

    use super::*;

    #[test]
    fn test_computation() -> Result<()> {
        snapshot_runner::test_snapshots("computation", |input| {
            let objects = input.get_json("objects").expect("could not get objects");

            let computation: ComputationSource = input
                .get_json("computation")
                .expect("could not get computation");
            let engine = ComputationEngine::Join { objects: &objects };

            let value = engine.compute(&computation).expect("could not compute");
            serde_json::to_string_sorted_pretty(&value).expect("could not serialize")
        })
    }
}

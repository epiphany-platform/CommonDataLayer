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
    use maplit::*;
    use serde_json::json;
    use test_case::test_case;
    use uuid::Uuid;

    use super::*;

    #[test_case(json!({"a": 2}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 0, "field_path": "a" }}"# => json!(2); "compute base schema field value")]
    #[test_case(json!({"a": {"aa": 58}}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 0, "field_path": "a.aa" }}"# => json!(58); "compute base schema inner field value")]
    #[test_case(json!({"a": 58}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 1, "field_path": "a" }}"# => json!(58); "compute joined field value")]
    #[test_case(json!({"a": {"aa": 58}}), json!({"b": 42}), r#"{"RawValue": { "value": "aaa" }}"# => json!("aaa"); "compute raw value")]
    fn calculates_value(
        object_value: Value,
        subobject_value: Value,
        computation_str: &str,
    ) -> Value {
        let computation: ComputationSource = serde_json::from_str(computation_str).unwrap();
        let object = ObjectIdPair {
            schema_id: Uuid::new_v4(),
            object_id: Uuid::new_v4(),
        };
        let subobject = ObjectIdPair {
            schema_id: Uuid::new_v4(),
            object_id: Uuid::new_v4(),
        };
        let objects = hashmap! {
            object => object_value,
            subobject => subobject_value
        };

        let engine = ComputationEngine::Join { objects: &objects };

        engine.compute(&computation).unwrap()
    }
}

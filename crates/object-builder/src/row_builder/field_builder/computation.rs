use std::collections::HashMap;

use anyhow::Result;
use cdl_dto::{
    edges::TreeObject,
    materialization::{
        Computation, EqualsComputation, FieldValueComputation, FullView, RawValueComputation,
    },
};
use serde_json::Value;

use crate::{
    row_builder::utils::{get_object_id_for_relation, get_sub_object},
    ObjectIdPair,
};

use super::FieldBuilder;

#[derive(Clone, Copy)]
pub struct ComputationEngine<'a> {
    object_pair: ObjectIdPair,
    objects: &'a HashMap<ObjectIdPair, Value>,
    tree_object: &'a TreeObject,
    view: &'a FullView,
}

impl<'a> ComputationEngine<'a> {
    pub fn compute(self, computation: &Computation) -> Result<Value> {
        Ok(match computation {
            Computation::RawValue(RawValueComputation { value }) => value.0.clone(),
            Computation::FieldValue(FieldValueComputation {
                schema_id,
                field_path,
            }) => {
                let object_id_pair = get_object_id_for_relation(
                    *schema_id,
                    self.object_pair,
                    self.view,
                    &self.tree_object,
                )?;
                let field_path_parts = field_path.split('.');
                let object = self.objects.get(&object_id_pair).unwrap();
                get_sub_object(object, field_path_parts)?
            }
            Computation::Equals(EqualsComputation { lhs, rhs }) => {
                let lhs = self.compute(lhs)?;
                let rhs = self.compute(rhs)?;
                Value::Bool(lhs == rhs)
            }
        })
    }
}

impl<'a> From<FieldBuilder<'a>> for ComputationEngine<'a> {
    fn from(fb: FieldBuilder<'a>) -> Self {
        let FieldBuilder {
            object_pair,
            objects,
            tree_object,
            view,
        } = fb;
        Self {
            object_pair,
            objects,
            tree_object,
            view,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use cdl_dto::{
        edges::SchemaRelation,
        materialization::{FieldDefinition, FieldType, Relation},
    };
    use maplit::*;
    use rpc::schema_registry::types::SearchFor;
    use serde_json::json;
    use test_case::test_case;
    use uuid::Uuid;

    use super::*;

    fn default_tree_object(object: ObjectIdPair, subobject: ObjectIdPair) -> TreeObject {
        TreeObject {
            object_id: object.object_id,
            relation_id: Uuid::new_v4(),
            relation: SchemaRelation {
                child_schema_id: subobject.schema_id,
                parent_schema_id: object.schema_id,
            },
            children: vec![subobject.object_id],
            subtrees: vec![],
        }
    }

    fn default_view(
        object: ObjectIdPair,
        tree_object: &TreeObject,
        computation: &Computation,
    ) -> FullView {
        FullView {
            id: Uuid::new_v4(),
            base_schema_id: object.schema_id,
            name: "".into(),
            materializer_address: "".into(),
            materializer_options: json!({}),
            fields: hashmap! {
                "foo".into() => FieldDefinition::Computed { field_type: FieldType::Numeric, computation: computation.clone() }
            },
            relations: vec![Relation {
                global_id: tree_object.relation_id,
                local_id: NonZeroU8::new(1).unwrap(),
                relations: vec![],
                search_for: SearchFor::Parents,
            }],
        }
    }

    #[test_case(json!({"a": 2}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 0, "field_path": "a" }}"# => json!(2); "compute base schema field value")]
    #[test_case(json!({"a": {"aa": 58}}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 0, "field_path": "a.aa" }}"# => json!(58); "compute base schema inner field value")]
    #[test_case(json!({"a": 58}), json!({"b": 42}), r#"{"FieldValue": { "schema_id": 1, "field_path": "a" }}"# => json!(58); "compute joined field value")]
    #[test_case(json!({"a": {"aa": 58}}), json!({"b": 42}), r#"{"RawValue": { "value": "aaa" }}"# => json!("aaa"); "compute raw value")]
    fn calculates_value(
        object_value: Value,
        subobject_value: Value,
        computation_str: &str,
    ) -> Value {
        let computation: Computation = serde_json::from_str(computation_str).unwrap();
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
        let tree_object = default_tree_object(object, subobject);

        let view = default_view(object, &tree_object, &computation);

        let engine = ComputationEngine {
            object_pair: object,
            objects: &objects,
            tree_object: &tree_object,
            view: &view,
        };

        engine.compute(&computation).unwrap()
    }
}

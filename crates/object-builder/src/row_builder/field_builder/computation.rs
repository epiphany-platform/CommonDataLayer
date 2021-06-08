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

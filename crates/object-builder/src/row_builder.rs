use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use cdl_dto::edges::TreeObject;
use cdl_dto::materialization::{FieldDefinition, FullView};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    buffer::{ObjectIdPair, RowSource},
    RowDefinition,
};

mod field_builder;
mod utils;

use field_builder::FieldBuilder;

#[derive(Clone, Copy)]
pub struct RowBuilder<'a> {
    view: &'a FullView,
}

impl<'a> RowBuilder<'a> {
    pub fn new(view: &'a FullView) -> Self {
        Self { view }
    }

    pub(crate) fn build(self, source: RowSource) -> Result<RowDefinition> {
        match source {
            RowSource::Join {
                objects,
                tree_object,
            } => self.build_join(objects, tree_object),
            RowSource::Single { object_pair, value } => self.build_single(object_pair, value),
        }
    }

    fn build_join(
        self,
        objects: HashMap<ObjectIdPair, Value>,
        tree_object: TreeObject,
    ) -> Result<RowDefinition> {
        let base_object_id: Uuid = tree_object.object_id;
        let base_schema_id: Uuid = tree_object.relation.parent_schema_id;
        let base_object_id_pair = ObjectIdPair {
            schema_id: base_schema_id,
            object_id: base_object_id,
        };

        let field_builder = FieldBuilder {
            object_pair: base_object_id_pair,
            objects: &objects,
            tree_object: &tree_object,
            view: self.view,
        };

        let fields = self
            .view
            .fields
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
        self,
        ObjectIdPair { object_id, .. }: ObjectIdPair,
        object_value: Value,
    ) -> Result<RowDefinition> {
        let object = object_value
            .as_object()
            .with_context(|| format!("Expected object ({}) to be a JSON object", object_id))?;

        use FieldDefinition::*;

        let fields = self
            .view
            .fields
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
                                    object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                        Computed { .. } => {
                            // TODO: In theory we could enable subset of it later
                            anyhow::bail!(
                                "Computed field definition is not supported in relation-less view"
                            )
                        }
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
        object_ids.insert(object_id);
        Ok(RowDefinition { object_ids, fields })
    }
}

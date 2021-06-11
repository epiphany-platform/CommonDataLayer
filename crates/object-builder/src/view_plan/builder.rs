use anyhow::{Context, Result};
use cdl_dto::{
    edges::TreeObject,
    materialization::{
        Computation, EqualsComputation, FieldDefinition, FieldValueComputation, FullView,
        RawValueComputation,
    },
};
use maplit::hashset;
use std::collections::HashSet;
use std::num::NonZeroU8;

use cdl_dto::materialization::{LocalId, Relation};
use rpc::schema_registry::types::SearchFor;
use uuid::Uuid;

use super::{UnfinishedRow, UnfinishedRowPair};
use crate::utils::get_base_object;
use crate::{ArrayElementSource, ComputationSource, FieldDefinitionSource, ObjectIdPair};

pub struct ViewPlanBuilder<'a> {
    pub view: &'a FullView,
}

impl<'a> ViewPlanBuilder<'a> {
    pub fn prepare_unfinished_rows<'b>(
        &self,
        root_object: ObjectIdPair,
        mut fields: impl Iterator<Item = (&'b String, &'b FieldDefinition)>,
        tree_obj: Option<&TreeObject>,
    ) -> Result<Vec<(UnfinishedRow, HashSet<ObjectIdPair>)>> {
        let unfinished_pair = UnfinishedRow::new(root_object);
        let mut pairs = vec![unfinished_pair];
        loop {
            if let Some((field_name, field)) = fields.next() {
                pairs = pairs
                    .into_iter()
                    .map(|pair| self.prepare_unfinished_row(pair, field_name, field, tree_obj))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten()
                    .collect();
            } else {
                break Ok(pairs);
            }
        }
    }

    fn prepare_computation(
        &self,
        pair: &UnfinishedRowPair,
        computation: &Computation,
        tree_obj: Option<&TreeObject>,
    ) -> Result<Vec<(ComputationSource, HashSet<ObjectIdPair>)>> {
        let base_object = pair.0.root_object;
        Ok(match computation {
            Computation::RawValue(RawValueComputation { value }) => {
                vec![(
                    ComputationSource::RawValue {
                        value: value.0.clone(),
                    },
                    hashset![],
                )]
            }
            Computation::FieldValue(FieldValueComputation {
                schema_id,
                field_path,
            }) => {
                let objects = self
                    .get_objects_ids_for_relation(*schema_id, tree_obj)?
                    .unwrap_or_else(|| vec![base_object]);

                objects
                    .into_iter()
                    .map(|object| {
                        let computation = ComputationSource::FieldValue {
                            object,
                            field_path: field_path.clone(),
                        };
                        (computation, hashset!(object, base_object))
                    })
                    .collect()
            }
            Computation::Equals(EqualsComputation { lhs, rhs }) => {
                let lhs = self.prepare_computation(pair, lhs, tree_obj)?;
                let rhs = self.prepare_computation(pair, rhs, tree_obj)?;

                lhs.into_iter()
                    .flat_map(|(lhs, lhs_objects)| {
                        rhs.iter().map(move |(rhs, rhs_objects)| {
                            let set = &lhs_objects | rhs_objects;
                            (
                                ComputationSource::Equals {
                                    lhs: Box::new(lhs.clone()),
                                    rhs: Box::new(rhs.clone()),
                                },
                                set,
                            )
                        })
                    })
                    .collect()
            }
        })
    }

    fn prepare_unfinished_row(
        &self,
        mut pair: UnfinishedRowPair,
        field_name: &str,
        field: &FieldDefinition,
        tree_obj: Option<&TreeObject>,
    ) -> Result<Vec<UnfinishedRowPair>> {
        let base_object = pair.0.root_object;
        match field {
            FieldDefinition::Simple {
                field_name,
                field_type,
            } => {
                let field = FieldDefinitionSource::Simple {
                    object: base_object,
                    field_name: field_name.clone(),
                    field_type: field_type.clone(),
                };
                pair.0.fields.insert(field_name.into(), field);
                pair.1.insert(base_object);
                Ok(vec![pair])
            }
            FieldDefinition::Computed {
                computation,
                field_type,
            } => {
                let computations = self.prepare_computation(&pair, computation, tree_obj)?;
                Ok(computations
                    .into_iter()
                    .map(|(computation, object_ids)| {
                        let mut pair = pair.clone();
                        let field = FieldDefinitionSource::Computed {
                            computation,
                            field_type: field_type.clone(),
                        };
                        pair.0.fields.insert(field_name.into(), field);
                        pair.1.extend(object_ids.into_iter());
                        pair
                    })
                    .collect())
            }
            FieldDefinition::Array { base, fields } => {
                let tree_object = self
                    .find_tree_object_for_relation(*base, tree_obj)?
                    .context("Array field type needs a reference to relation in view definition")?;

                let (elements, objects): (Vec<_>, Vec<_>) = self
                    .prepare_unfinished_rows(
                        get_base_object(tree_object),
                        fields.iter(),
                        Some(tree_object),
                    )?
                    .into_iter()
                    .map(|(row, objects)| (ArrayElementSource { fields: row.fields }, objects))
                    .unzip();

                let objects = objects.into_iter().flatten();

                let field = FieldDefinitionSource::Array { elements };
                pair.0.fields.insert(field_name.into(), field);
                pair.1.extend(objects);
                Ok(vec![pair])
            }
        }
    }

    fn find_tree_object_for_relation(
        &self,
        relation_id: LocalId,
        tree_object: Option<&'a TreeObject>,
    ) -> Result<Option<&'a TreeObject>> {
        Ok(match NonZeroU8::new(relation_id) {
            None => None,
            Some(relation_id) => {
                let relation = self
                    .view
                    .relations
                    .iter()
                    .find_map(|r| find_relation(r, relation_id))
                    .with_context(|| format!("Could not find a relation: {}", relation_id))?;

                let global_id = relation.global_id;
                let tree_object =
                    tree_object.context("Trying to retrieve relation but got no edges")?;
                let tree_object = find_tree_object(tree_object, global_id).with_context(|| {
                    format!(
                        "Could not find a relation in edge registry: {}",
                        relation_id
                    )
                })?;
                Some(tree_object)
            }
        })
    }

    fn get_objects_ids_for_relation(
        &self,
        relation_id: LocalId,
        tree_object: Option<&TreeObject>,
    ) -> Result<Option<Vec<ObjectIdPair>>> {
        let relation_id = NonZeroU8::new(relation_id);
        Ok(match relation_id {
            None => None,
            Some(relation_id) => {
                let relation = self
                    .view
                    .relations
                    .iter()
                    .find_map(|r| find_relation(r, relation_id))
                    .with_context(|| format!("Could not find a relation: {}", relation_id))?;

                let global_id = relation.global_id;
                let tree_object =
                    tree_object.context("Trying to retrieve relation but got no edges")?;
                let tree_object = find_tree_object(tree_object, global_id).with_context(|| {
                    format!(
                        "Could not find a relation in edge registry: {}",
                        relation_id
                    )
                })?;

                Some(match relation.search_for {
                    SearchFor::Children => {
                        let schema_id = tree_object.relation.child_schema_id;

                        let children = tree_object
                            .children
                            .iter()
                            .map(|object_id| ObjectIdPair {
                                schema_id,
                                object_id: *object_id,
                            })
                            .collect::<Vec<_>>();

                        children
                    }
                    SearchFor::Parents => vec![ObjectIdPair {
                        schema_id: tree_object.relation.parent_schema_id,
                        object_id: tree_object.object_id,
                    }],
                })
            }
        })
    }
}

fn find_relation(relation: &Relation, relation_id: NonZeroU8) -> Option<&Relation> {
    if relation.local_id == relation_id {
        return Some(relation);
    }
    relation
        .relations
        .iter()
        .find_map(|r| find_relation(r, relation_id))
}

fn find_tree_object(tree_object: &TreeObject, global_id: Uuid) -> Option<&TreeObject> {
    if tree_object.relation_id == global_id {
        return Some(tree_object);
    }

    tree_object
        .subtrees
        .iter()
        .flat_map(|r| r.objects.iter())
        .find_map(|t| find_tree_object(t, global_id))
}

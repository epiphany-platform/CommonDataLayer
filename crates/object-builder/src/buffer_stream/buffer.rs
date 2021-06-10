use anyhow::{Context, Result};
use cdl_dto::{
    edges::{TreeObject, TreeResponse},
    materialization::{
        Computation, EqualsComputation, FieldDefinition, FieldValueComputation, FullView,
        RawValueComputation,
    },
};
use maplit::hashset;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU8,
};

use crate::utils::{find_tree_object_for_relation, get_objects_ids_for_relation};
use crate::{
    ArrayElementSource, ComputationSource, FieldDefinitionSource, ObjectIdPair, RowSource,
};

/// Because objects are received on the go, and object builder needs to create joins,
/// these objects need to be tempoirairly stored in some kind of buffer until the last part
/// of the join arrives
pub struct ObjectBuffer {
    unfinished_rows: Vec<Option<UnfinishedRow>>,
    missing: HashMap<ObjectIdPair, Vec<usize>>, // (_, indices to unfinished_rows)
    view: FullView,
}

type UnfinishedRowPair = (UnfinishedRow, HashSet<ObjectIdPair>);

#[derive(Clone)]
struct UnfinishedRow {
    /// Number of objects that are still missing to finish the join
    missing: usize,
    /// Stored objects waiting for missing ones
    objects: HashMap<ObjectIdPair, Value>,

    root_object: ObjectIdPair,

    fields: HashMap<String, FieldDefinitionSource>,
}

impl UnfinishedRow {
    fn new(root_object: ObjectIdPair) -> UnfinishedRowPair {
        (
            Self {
                missing: 0,
                objects: Default::default(),
                root_object,
                fields: Default::default(),
            },
            hashset![],
        )
    }

    fn into_single(self, value: Value) -> RowSource {
        RowSource::Single {
            root_object: self.root_object,
            value,
            fields: self.fields,
        }
    }
    fn into_join(self) -> RowSource {
        RowSource::Join {
            objects: self.objects,
            root_object: self.root_object,
            // tree_object: self.tree_object,
            fields: self.fields,
        }
    }
}

fn get_base_object(tree_object: &TreeObject) -> ObjectIdPair {
    let object_id = tree_object.object_id;
    let schema_id = tree_object.relation.parent_schema_id;
    ObjectIdPair {
        object_id,
        schema_id,
    }
}

fn prepare_unfinished_row(
    mut pair: UnfinishedRowPair,
    field_name: &str,
    field: &FieldDefinition,
    tree_obj: Option<&TreeObject>,
    view: &FullView,
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
            fn prepare_computation(
                pair: &UnfinishedRowPair,
                computation: &Computation,
                tree_obj: Option<&TreeObject>,
                view: &FullView,
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
                        let objects = get_objects_ids_for_relation(*schema_id, view, tree_obj)?
                            .unwrap_or_else(|| vec![base_object]);

                        objects
                            .into_iter()
                            .map(|object| {
                                let computation = ComputationSource::FieldValue {
                                    object,
                                    field_path: field_path.clone(),
                                };
                                (computation, hashset!(object))
                            })
                            .collect()
                    }
                    Computation::Equals(EqualsComputation { lhs, rhs }) => {
                        let lhs = prepare_computation(pair, lhs, tree_obj, view)?;
                        let rhs = prepare_computation(pair, rhs, tree_obj, view)?;

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

            let computations = prepare_computation(&pair, computation, tree_obj, view)?;
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
            let tree_object = find_tree_object_for_relation(*base, view, tree_obj)?
                .context("Array field type needs a reference to relation in view definition")?;

            let (elements, objects): (Vec<_>, Vec<_>) = prepare_unfinished_rows(
                get_base_object(tree_object),
                fields.iter(),
                Some(tree_object),
                view,
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

fn prepare_unfinished_rows<'a>(
    root_object: ObjectIdPair,
    mut fields: impl Iterator<Item = (&'a String, &'a FieldDefinition)>,
    tree_obj: Option<&TreeObject>,
    view: &FullView,
) -> Result<Vec<(UnfinishedRow, HashSet<ObjectIdPair>)>> {
    let unfinished_pair = UnfinishedRow::new(root_object);
    let mut pairs = vec![unfinished_pair];
    loop {
        if let Some((field_name, field)) = fields.next() {
            pairs = pairs
                .into_iter()
                .map(|pair| prepare_unfinished_row(pair, field_name, field, tree_obj, view))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .map(|(mut row, set)| {
                    row.missing = set.len();
                    (row, set)
                })
                .filter(|(row, _)| row.missing > 0)
                .collect();
        } else {
            break Ok(pairs);
        }
    }
}

impl ObjectBuffer {
    pub fn try_new(view: FullView, edges: &HashMap<NonZeroU8, TreeResponse>) -> Result<Self> {
        let mut missing: HashMap<ObjectIdPair, Vec<usize>> = Default::default();

        let unfinished_rows = edges
            .values()
            .flat_map(|res| res.objects.iter())
            .map(|tree_object| {
                prepare_unfinished_rows(
                    get_base_object(tree_object),
                    view.fields.iter(),
                    Some(tree_object),
                    &view,
                )
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(idx, (row, set))| {
                for object in set {
                    missing.entry(object).or_default().push(idx);
                }
                Some(row)
            })
            .collect();

        Ok(Self {
            unfinished_rows,
            missing,
            view,
        })
    }

    pub fn add_object(
        &mut self,
        pair: ObjectIdPair,
        value: Value,
    ) -> Option<Result<Vec<RowSource>>> {
        match self.missing.get_mut(&pair) {
            Some(missing_indices) => {
                if missing_indices.is_empty() {
                    tracing::error!("Got unexpected object: {}. Skipping...", pair.object_id);
                    return None;
                }
                let mut result = vec![];
                for missing_idx in missing_indices {
                    let unfinished_row_opt = self.unfinished_rows.get_mut(*missing_idx).unwrap();
                    let unfinished_row = unfinished_row_opt.as_mut()?;
                    unfinished_row.missing =
                        unfinished_row.missing.checked_sub(1).unwrap_or_default();
                    unfinished_row.objects.insert(pair, value.clone());
                    if unfinished_row.missing == 0 {
                        let row = std::mem::take(unfinished_row_opt)?; // Cant remove it because it would invalidate indices
                        result.push(row.into_join());
                    }
                }
                if result.is_empty() {
                    None
                } else {
                    Some(Ok(result))
                }
            }
            None => {
                // if we are processing object that is not missing it means either:
                // 1. there are no relations defined for this view and we should just process it
                // 2. we got an object_id event when we didnt ask for it.
                //      remember - object builder asks for specific object ids
                // Therefore only 1. option should be possible and we should return it
                let rows = prepare_unfinished_rows(pair, self.view.fields.iter(), None, &self.view);

                Some(rows.map(|rows| {
                    rows.into_iter()
                        .map(|(row, _)| row.into_single(value.clone()))
                        .collect()
                }))
            }
        }
    }
}

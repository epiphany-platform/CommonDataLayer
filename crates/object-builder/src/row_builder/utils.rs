use std::num::NonZeroU8;

use anyhow::{Context, Result};
use cdl_dto::{
    edges::TreeObject,
    materialization::{FullView, LocalId, Relation},
};
use rpc::schema_registry::types::SearchFor;
use serde_json::Value;
use uuid::Uuid;

use crate::ObjectIdPair;

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

pub fn get_sub_object(value: &Value, mut path: std::str::Split<char>) -> Result<Value> {
    match path.next() {
        Some(next) => {
            let object = value
                .as_object()
                .with_context(|| format!("Expected `{}` to be a JSON object", next))?;
            let sub_object = object
                .get(next)
                .with_context(|| format!("Field `{}` is missing", next))?;

            get_sub_object(sub_object, path)
        }
        None => Ok(value.clone()),
    }
}

pub fn get_object_id_for_relation(
    relation_id: LocalId,
    base_pair: ObjectIdPair,
    view: &cdl_dto::materialization::FullView,
    tree_object: &TreeObject,
) -> anyhow::Result<ObjectIdPair> {
    let relation_id = NonZeroU8::new(relation_id);

    match relation_id {
        None => Ok(base_pair),
        Some(relation_id) => {
            let relation = view
                .relations
                .iter()
                .find_map(|r| find_relation(r, relation_id))
                .with_context(|| format!("Could not find a relation: {}", relation_id))?;

            let global_id = relation.global_id;
            match relation.search_for {
                SearchFor::Parents => {
                    let tree_object =
                        find_tree_object(tree_object, global_id).with_context(|| {
                            format!(
                                "Could not find a relation in edge registry: {}",
                                relation_id
                            )
                        })?;

                    Ok(ObjectIdPair {
                        schema_id: tree_object.relation.parent_schema_id,
                        object_id: tree_object.object_id,
                    })
                }
                SearchFor::Children => {
                    // TODO: Send error to materializer so it can do transaction rollup #543
                    anyhow::bail!("Could not retrieve single row.")
                }
            }
        }
    }
}

pub fn get_objects_ids_for_relation(
    relation_id: LocalId,
    view: &FullView,
    tree_object: &TreeObject,
) -> Result<Vec<ObjectIdPair>> {
    let relation_id = NonZeroU8::new(relation_id);
    match relation_id {
        None => anyhow::bail!(
            "Array field definition must points into relation id (value higher than zero)"
        ),
        Some(relation_id) => {
            let relation = view
                .relations
                .iter()
                .find_map(|r| find_relation(r, relation_id))
                .with_context(|| format!("Could not find a relation: {}", relation_id))?;

            let global_id = relation.global_id;
            let tree_object = find_tree_object(tree_object, global_id).with_context(|| {
                format!(
                    "Could not find a relation in edge registry: {}",
                    relation_id
                )
            })?;

            match relation.search_for {
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

                    Ok(children)
                }
                SearchFor::Parents => Ok(vec![ObjectIdPair {
                    schema_id: tree_object.relation.parent_schema_id,
                    object_id: tree_object.object_id,
                }]),
            }
        }
    }
}

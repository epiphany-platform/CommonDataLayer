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

pub fn find_tree_object_for_relation<'a>(
    relation_id: LocalId,
    view: &FullView,
    tree_object: Option<&'a TreeObject>,
) -> Result<Option<&'a TreeObject>> {
    Ok(match NonZeroU8::new(relation_id) {
        None => None,
        Some(relation_id) => {
            let relation = view
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

pub fn get_objects_ids_for_relation(
    relation_id: LocalId,
    view: &FullView,
    tree_object: Option<&TreeObject>,
) -> Result<Option<Vec<ObjectIdPair>>> {
    let relation_id = NonZeroU8::new(relation_id);
    Ok(match relation_id {
        None => None,
        Some(relation_id) => {
            let relation = view
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

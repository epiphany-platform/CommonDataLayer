use anyhow::{Context, Result};
use cdl_dto::edges::TreeObject;
use serde_json::Value;

use crate::ObjectIdPair;

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

pub fn get_base_object(tree_object: &TreeObject) -> ObjectIdPair {
    let object_id = tree_object.object_id;
    let schema_id = tree_object.relation.parent_schema_id;
    ObjectIdPair {
        object_id,
        schema_id,
    }
}

use crate::{
    db::SchemaDb, error::RegistryResult, types::storage::vertices::Definition, types::VersionedUuid,
};
use indradb::Datastore;
use jsonschema::JSONSchema;
use serde_json::Value;
use uuid::Uuid;

pub fn build_full_schema<D: Datastore>(
    mut schema: Value,
    db: &SchemaDb<D>,
) -> RegistryResult<Definition> {
    if let Some(defs) = schema
        .get_mut("definitions")
        .and_then(|val| val.as_object_mut())
    {
        let schemas_to_retrieve: Vec<(String, Uuid)> = defs
            .iter()
            .filter_map(|(key, value)| {
                value
                    .as_str()
                    .and_then(|val| Uuid::parse_str(val).ok())
                    .map(|id| (key.clone(), id))
            })
            .collect();

        for (key, id) in schemas_to_retrieve {
            let schema = db.get_schema_definition(&VersionedUuid::any(id))?;
            defs[&key] = schema.definition;
        }
    }

    JSONSchema::compile(&schema)?;

    Ok(Definition { definition: schema })
}

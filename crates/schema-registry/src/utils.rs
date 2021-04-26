use crate::error::{RegistryError, RegistryResult};
use crate::{db::SchemaRegistryDb, types::VersionedUuid};
use jsonschema::JSONSchema;
use serde_json::Value;
use uuid::Uuid;

pub async fn build_full_schema(schema: &mut Value, conn: &SchemaRegistryDb) -> RegistryResult<()> {
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
            let (_version, definition) =
                conn.get_schema_definition(&VersionedUuid::any(id)).await?;
            defs[&key] = definition;
        }
    }

    JSONSchema::compile(&schema).map_err(RegistryError::InvalidJsonSchema)?;

    Ok(())
}

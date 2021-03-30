use std::convert::TryInto;
use std::path::PathBuf;

use semver::{Version, VersionReq};
use serde_json::Value;
use uuid::Uuid;

use crate::utils::*;
use rpc::schema_registry::{
    types::SchemaType, Empty, Id, NewSchema, NewSchemaVersion, SchemaDefinition, SchemaMetadata,
    SchemaMetadataPatch, SchemaMetadataUpdate, ValueToValidate, VersionedId,
};

pub async fn get_schema_definition(
    schema_id: Uuid,
    version: Option<VersionReq>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let response = client
        .get_schema_definition(VersionedId {
            id: schema_id.to_string(),
            version_req: version.map(|v| v.to_string()),
        })
        .await?;

    println!(
        "{:#}",
        serde_json::from_slice::<Value>(&response.into_inner().definition)?
    );

    Ok(())
}

pub async fn add_schema(
    schema_name: String,
    insert_destination: String,
    query_address: String,
    file: Option<PathBuf>,
    registry_addr: String,
    schema_type: SchemaType,
) -> anyhow::Result<()> {
    let definition = read_json(file)?;

    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let response = client
        .add_schema(NewSchema {
            metadata: SchemaMetadata {
                name: schema_name.clone(),
                query_address,
                insert_destination,
                schema_type: schema_type.into(),
            },
            definition: serde_json::to_vec(&definition)?,
        })
        .await?;

    eprintln!(
        "Successfully added schema \"{}\" to the schema registry.",
        schema_name
    );
    eprintln!("The following UUID was assigned:");
    println!("{}", response.into_inner().id);

    Ok(())
}

pub async fn get_schema_versions(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let response = client
        .get_schema_versions(Id {
            id: schema_id.to_string(),
        })
        .await?;

    let mut versions = response
        .into_inner()
        .versions
        .into_iter()
        .map(|v: String| Version::parse(&v))
        .collect::<Result<Vec<Version>, _>>()?;
    versions.sort();

    for version in versions {
        println!("{}", version);
    }

    Ok(())
}

pub async fn update_schema(
    id: Uuid,
    name: Option<String>,
    insert_destination: Option<String>,
    query_address: Option<String>,
    schema_type: Option<SchemaType>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    client
        .update_schema(SchemaMetadataUpdate {
            id: id.to_string(),
            patch: SchemaMetadataPatch {
                name,
                insert_destination,
                query_address,
                schema_type: schema_type.map(|t| t.into()),
            },
        })
        .await?;

    Ok(())
}

pub async fn add_schema_version(
    schema_id: Uuid,
    version: Version,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let definition = read_json(file)?;

    let schema = NewSchemaVersion {
        id: schema_id.to_string(),
        definition: SchemaDefinition {
            version: version.to_string(),
            definition: serde_json::to_vec(&definition)?,
        },
    };

    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    client.add_schema_version(schema).await?;

    Ok(())
}

pub async fn get_schema_names(registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let mut schemas = client.get_all_schemas(Empty {}).await?.into_inner().schemas;

    if schemas.is_empty() {
        anyhow::bail!("No schemas exist yet in the schema registry.");
    }

    schemas.sort_by_key(|schema| schema.metadata.name.clone());
    for schema in schemas {
        println!("ID: {}, Name: {}", schema.id, schema.metadata.name);
    }

    Ok(())
}

pub async fn get_schema_metadata(id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let metadata = client
        .get_schema_metadata(Id { id: id.to_string() })
        .await?
        .into_inner();
    let schema_type: SchemaType = metadata.schema_type.try_into()?;

    println!("Name: {}", metadata.name);
    println!("Topic or Queue: {}", metadata.insert_destination);
    println!("Query Address: {}", metadata.query_address);
    println!("Type: {}", schema_type);

    Ok(())
}

pub async fn validate_value(
    schema_id: Uuid,
    version_req: Option<VersionReq>,
    file: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let value = read_json(file)?;

    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let errors = client
        .validate_value(ValueToValidate {
            schema_id: VersionedId {
                id: schema_id.to_string(),
                version_req: version_req.map(|v| v.to_string()),
            },
            value: serde_json::to_vec(&value)?,
        })
        .await?
        .into_inner()
        .errors;

    if !errors.is_empty() {
        anyhow::bail!("The value is not valid. {}", errors.join(", "));
    }

    println!("The value is valid.");
    Ok(())
}

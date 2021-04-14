use std::collections::HashMap;
use std::path::PathBuf;

use rpc::schema_registry::{Id, NewView, ViewUpdate};
use uuid::Uuid;

use crate::utils::read_json;

pub async fn get_view(view_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let view = client
        .get_view(Id {
            id: view_id.to_string(),
        })
        .await?
        .into_inner();

    println!(
        "Name: {}, Materializer Address: {}",
        view.name, view.materializer_address
    );

    Ok(())
}

pub async fn add_view_to_schema(
    schema_id: Uuid,
    name: String,
    materializer_address: String,
    materializer_options: String,
    fields: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;

    let fields = read_json(fields)?;
    let view = NewView {
        schema_id: schema_id.to_string(),
        name: name.clone(),
        materializer_address,
        materializer_options,
        fields,
    };

    let response = client.add_view_to_schema(view).await?;

    eprintln!(
        "Successfully added view \"{}\" to schema \"{}\" in the schema registry.",
        name, schema_id
    );
    eprintln!("The following UUID was assigned:");
    println!("{}", response.into_inner().id);

    Ok(())
}

pub async fn update_view(
    view_id: Uuid,
    name: Option<String>,
    materializer_address: Option<String>,
    materializer_options: Option<String>,
    fields: Option<PathBuf>,
    update_fields: bool,
    materializer_addr: Option<String>,
    fields: Option<String>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;

    let fields = if update_fields {
        serde_json::from_value(read_json(fields)?)?
    } else {
        HashMap::default()
    };
    let view = ViewUpdate {
        id: view_id.to_string(),
        name,
        materializer_address,
        materializer_options,
        update_fields,
        fields,
    };

    client.update_view(view).await?.into_inner();

    eprintln!("Successfully updated view.");

    Ok(())
}

pub async fn get_schema_views(schema_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let views = client
        .get_all_views_of_schema(Id {
            id: schema_id.to_string(),
        })
        .await?
        .into_inner()
        .views;

    if views.is_empty() {
        anyhow::bail!("No views exist yet for the given schema in the schema registry.");
    }

    for view in views {
        println!("ID: {}, Name: {}", view.id, view.name);
    }

    Ok(())
}

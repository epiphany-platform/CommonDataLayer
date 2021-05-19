use std::path::PathBuf;

use cdl_dto::materialization::{Filter, Relation};
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

#[allow(clippy::clippy::too_many_arguments)]
pub async fn add_view_to_schema(
    base_schema_id: Uuid,
    name: String,
    materializer_address: String,
    materializer_options: String,
    fields: Option<PathBuf>,
    filters: Option<PathBuf>,
    relations: Option<PathBuf>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;

    let fields = read_json(fields)?;
    let filters: Option<Filter> = serde_json::from_value(read_json(filters)?)?;
    let relations: Vec<Relation> = serde_json::from_value(read_json(relations)?)?;

    let view = NewView {
        base_schema_id: base_schema_id.to_string(),
        name: name.clone(),
        materializer_address,
        materializer_options,
        fields: serde_json::from_value(fields)?,
        filters: filters.map(|f| f.into_rpc()).transpose()?,
        relations: relations.into_iter().map(|r| r.into_rpc()).collect(),
    };

    let response = client.add_view_to_schema(view).await?;

    eprintln!(
        "Successfully added view \"{}\" to schema \"{}\" in the schema registry.",
        name, base_schema_id
    );
    eprintln!("The following UUID was assigned:");
    println!("{}", response.into_inner().id);

    Ok(())
}

#[allow(clippy::clippy::too_many_arguments)]
pub async fn update_view(
    view_id: Uuid,
    name: Option<String>,
    materializer_address: Option<String>,
    materializer_options: Option<String>,
    fields: Option<PathBuf>,
    update_fields: bool,
    filters: Option<PathBuf>,
    update_filters: bool,
    relations: Option<PathBuf>,
    update_relations: bool,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;

    let filters = if update_filters {
        let filter: Option<Filter> = serde_json::from_value(read_json(filters)?)?;
        filter.map(|f| f.into_rpc()).transpose()?
    } else {
        Default::default()
    };

    let relations = if update_relations {
        let relation: Vec<Relation> = serde_json::from_value(read_json(relations)?)?;
        relation.into_iter().map(|r| r.into_rpc()).collect()
    } else {
        Default::default()
    };

    let fields = if update_fields {
        serde_json::from_value(read_json(fields)?)?
    } else {
        Default::default()
    };

    let view = ViewUpdate {
        id: view_id.to_string(),
        name,
        materializer_address,
        materializer_options: materializer_options.unwrap_or_default(),
        update_fields,
        fields,
        filters,
        update_filters,
        relations,
        update_relations,
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

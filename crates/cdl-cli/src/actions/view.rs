use rpc::schema_registry::{Id, NewSchemaView, UpdatedView};
use uuid::Uuid;

pub async fn get_view(view_id: Uuid, registry_addr: String) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let view = client
        .get_view(Id {
            id: view_id.to_string(),
        })
        .await?
        .into_inner();

    println!("Name: {}, JMESPath: {}", view.name, view.jmespath);

    Ok(())
}

pub async fn add_view_to_schema(
    schema_id: Uuid,
    name: String,
    jmespath: String,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let view = NewSchemaView {
        view_id: "".into(),
        schema_id: schema_id.to_string(),
        name: name.clone(),
        jmespath,
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
    jmespath: Option<String>,
    registry_addr: String,
) -> anyhow::Result<()> {
    let mut client = rpc::schema_registry::connect(registry_addr).await?;
    let view = UpdatedView {
        id: view_id.to_string(),
        name,
        jmespath,
    };

    let old_view = client.update_view(view).await?.into_inner();

    println!(
        "Old Name: {}, Old JMESPath: {}",
        old_view.name, old_view.jmespath
    );

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

    for (id, view) in views {
        println!("ID: {}, Name: {}", id, view.name);
    }

    Ok(())
}

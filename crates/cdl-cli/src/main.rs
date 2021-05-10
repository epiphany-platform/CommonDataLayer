pub mod actions;
pub mod args;
pub mod utils;

use actions::schema::*;
use actions::view::*;
use args::*;
use clap::Clap;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    ::utils::tracing::init(None, "cdl-cli")?;

    match args.action {
        Action::Schema { action } => match action {
            SchemaAction::Names => get_schema_names(args.registry_addr).await,
            SchemaAction::Definition { id, version } => {
                get_schema_definition(id, version, args.registry_addr).await
            }
            SchemaAction::Metadata { id } => get_schema_metadata(id, args.registry_addr).await,
            SchemaAction::Versions { id } => get_schema_versions(id, args.registry_addr).await,
            SchemaAction::Add {
                name,
                insert_destination,
                query_address,
                file,
                schema_type,
            } => {
                add_schema(
                    name,
                    insert_destination,
                    query_address,
                    file,
                    schema_type,
                    args.registry_addr,
                )
                .await
            }
            SchemaAction::AddVersion { id, version, file } => {
                add_schema_version(id, version, file, args.registry_addr).await
            }
            SchemaAction::Update {
                id,
                name,
                insert_destination,
                query_address,
                schema_type,
            } => {
                update_schema(
                    id,
                    name,
                    insert_destination,
                    query_address,
                    schema_type,
                    args.registry_addr,
                )
                .await
            }
            SchemaAction::Validate { id, version, file } => {
                validate_value(id, version, file, args.registry_addr).await
            }
        },
        Action::View { action } => match action {
            ViewAction::Names { schema_id } => {
                get_schema_views(schema_id, args.registry_addr).await
            }
            ViewAction::Get { id } => get_view(id, args.registry_addr).await,
            ViewAction::Add {
                schema_id,
                name,
                materializer_address,
                materializer_options,
                fields,
            } => {
                add_view_to_schema(
                    schema_id,
                    name,
                    materializer_address,
                    materializer_options,
                    fields,
                    args.registry_addr,
                )
                .await
            }
            ViewAction::Update {
                id,
                name,
                fields,
                update_fields,
                materializer_address,
                materializer_options,
            } => {
                update_view(
                    id,
                    name,
                    materializer_address,
                    materializer_options,
                    fields,
                    update_fields,
                    args.registry_addr,
                )
                .await
            }
        },
    }
}

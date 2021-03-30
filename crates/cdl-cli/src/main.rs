pub mod actions;
pub mod args;
pub mod utils;

use actions::schema::*;
use actions::view::*;
use args::*;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    ::utils::tracing::init();

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
                topic_or_queue,
                query_address,
                file,
                schema_type,
            } => {
                add_schema(
                    name,
                    topic_or_queue.unwrap_or_default(),
                    query_address.unwrap_or_default(),
                    file,
                    args.registry_addr,
                    schema_type,
                )
                .await
            }
            SchemaAction::AddVersion { id, version, file } => {
                add_schema_version(id, version, file, args.registry_addr).await
            }
            SchemaAction::Update {
                id,
                name,
                topic_or_queue,
                query_address,
                schema_type,
            } => {
                update_schema(
                    id,
                    name,
                    topic_or_queue,
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
                fields,
            } => {
                add_view_to_schema(
                    schema_id,
                    name,
                    materializer_address,
                    fields,
                    args.registry_addr,
                )
                .await
            }
            ViewAction::Update {
                id,
                name,
                materializer_address,
                fields,
                update_fields,
            } => {
                update_view(
                    id,
                    name,
                    materializer_address,
                    fields,
                    update_fields,
                    args.registry_addr,
                )
                .await
            }
        },
    }
}

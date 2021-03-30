pub mod actions;
pub mod args;
pub mod utils;

use actions::schema::*;
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
    }
}

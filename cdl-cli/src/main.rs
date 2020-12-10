pub mod actions;
pub mod args;
pub mod utils;

use actions::{schema::*, view::*};
use args::*;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    match args.action {
        Action::Schema { action } => match action {
            SchemaAction::Names => get_schema_names(args.registry_addr).await,
            SchemaAction::Get { schema_id, version } => {
                get_schema(schema_id, version, args.registry_addr).await
            }
            SchemaAction::GetTopic { schema_id } => {
                get_schema_topic(schema_id, args.registry_addr).await
            }
            SchemaAction::GetQueryAddress { schema_id } => {
                get_schema_query_address(schema_id, args.registry_addr).await
            }
            SchemaAction::GetSchemaType { schema_id } => {
                get_schema_type(schema_id, args.registry_addr).await
            }
            SchemaAction::Versions { schema_id } => {
                get_schema_versions(schema_id, args.registry_addr).await
            }
            SchemaAction::Add {
                name,
                topic,
                query_address,
                file,
                schema_type,
            } => {
                add_schema(
                    name,
                    topic,
                    query_address,
                    file,
                    args.registry_addr,
                    schema_type,
                )
                .await
            }
            SchemaAction::AddVersion {
                schema_id,
                version,
                file,
            } => add_schema_version(schema_id, version, file, args.registry_addr).await,
            SchemaAction::SetName { id, name } => {
                set_schema_name(id, name, args.registry_addr).await
            }
            SchemaAction::SetTopic { id, topic } => {
                set_schema_topic(id, topic, args.registry_addr).await
            }
            SchemaAction::SetQueryAddress { id, query_address } => {
                set_schema_query_address(id, query_address, args.registry_addr).await
            }
            SchemaAction::SetSchemaType { id, schema_type } => {
                set_schema_type(id, schema_type, args.registry_addr).await
            }
            SchemaAction::Validate { schema_id, file } => {
                validate_value(schema_id, file, args.registry_addr).await
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
                jmespath,
            } => add_view_to_schema(schema_id, name, jmespath, args.registry_addr).await,
            ViewAction::Update { id, name, jmespath } => {
                update_view(id, name, jmespath, args.registry_addr).await
            }
        },
    }
}

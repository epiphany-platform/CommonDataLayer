use rpc::schema_registry::types::SchemaType;
use semver::{Version, VersionReq};
use std::path::PathBuf;
use structopt::StructOpt;
use uuid::Uuid;

/// A tool to interact with services in the Common Data Layer.
#[derive(StructOpt)]
pub struct Args {
    // The address where the schema registry is hosted.
    #[structopt(long)]
    pub registry_addr: String,

    /// What to do for the provided schema.
    #[structopt(subcommand)]
    pub action: Action,
}

#[derive(StructOpt)]
pub enum Action {
    /// Work with the schemas in the schema registry.
    Schema {
        #[structopt(subcommand)]
        action: SchemaAction,
    },

    /// Work with views of schemas in the schema registry.
    View {
        #[structopt(subcommand)]
        action: ViewAction,
    },
}

#[derive(StructOpt)]
pub enum SchemaAction {
    /// Get the names and ids of all schemas currently stored in the schema
    /// registry, ordered alphabetically by name.
    Names,

    /// Get a schema from the registry and print it as JSON. By default, this
    /// retrieves the latest version, but you can pass a semver range to get
    /// a specific version.
    Get {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,

        /// An optional version requirement on the schema.
        #[structopt(short, long)]
        version: Option<VersionReq>,
    },

    /// Get a schema's kafka topic from the registry.
    GetTopic {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
    },

    /// Get a schema's query address from the registry.
    GetQueryAddress {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
    },

    /// Get a schema's type from the registry.
    GetSchemaType {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
    },

    /// List all semantic versions of a schema in the registry.
    Versions {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
    },

    /// Add a schema to the registry. It is assigned version `1.0.0`.
    Add {
        /// The name of the schema.
        #[structopt(short, long)]
        name: String,
        /// The topic of the schema.
        #[structopt(short, long, default_value = "")]
        topic: String,
        /// The query address of the schema.
        #[structopt(short, long, default_value = "")]
        query_address: String,
        /// The file containing the JSON Schema. If not provided,
        /// the schema will be read from stdin.
        #[structopt(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
        /// The type of schema. Possible values: DocumentStorage, Timeseries.
        #[structopt(short, long, default_value = "DocumentStorage")]
        schema_type: SchemaType,
    },

    /// Add a new version of an existing schema in the registry.
    AddVersion {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
        /// The new version of the schema. Must be greater than all existing versions.
        #[structopt(short, long)]
        version: Version,
        /// The file containing the JSON Schema. If not provided,
        /// the schema will be read from stdin.
        #[structopt(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },

    /// Update a schema's name in the registry.
    SetName {
        /// The id of the schema.
        #[structopt(short, long)]
        id: Uuid,
        /// The new name of the schema.
        #[structopt(short, long)]
        name: String,
    },

    /// Update a schema's topic in the registry.
    SetTopic {
        /// The id of the schema.
        #[structopt(short, long)]
        id: Uuid,
        /// The new topic of the schema.
        #[structopt(short, long)]
        topic: String,
    },

    /// Update a schema's query address in the registry.
    SetQueryAddress {
        /// The id of the schema.
        #[structopt(short, long)]
        id: Uuid,
        /// The new query address of the schema.
        #[structopt(short, long)]
        query_address: String,
    },

    /// Update a schema's type in the registry.
    SetSchemaType {
        /// The id of the schema.
        #[structopt(short, long)]
        id: Uuid,
        /// The new type of the schema. Possible values: DocumentStorage, Timeseries.
        #[structopt(short, long)]
        schema_type: SchemaType,
    },

    /// Validate that a JSON value is valid under the format of the
    /// given schema in the registry.
    Validate {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
        /// The file containing the JSON value. If not provided,
        /// the value will be read from stdin.
        #[structopt(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },
}

#[derive(StructOpt)]
pub enum ViewAction {
    /// Get the names of all views currently set on a schema,
    /// ordered alphabetically.
    Names {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
    },

    /// Get a view in the registry.
    Get {
        /// The id of the view.
        #[structopt(short, long)]
        id: Uuid,
    },

    /// Add a new view to a schema in the registry.
    Add {
        /// The id of the schema.
        #[structopt(short, long)]
        schema_id: Uuid,
        /// The name of the view.
        #[structopt(short, long)]
        name: String,
        /// The JMESPath the view will filter data with.
        #[structopt(short, long)]
        jmespath: String,
    },

    /// Update an existing view in the registry,
    /// and print the old view.
    Update {
        /// The id of the view.
        #[structopt(short, long)]
        id: Uuid,
        /// The new name of the view.
        #[structopt(short, long)]
        name: Option<String>,
        /// The new JMESPath the view will filter data with.
        #[structopt(short, long)]
        jmespath: Option<String>,
    },
}

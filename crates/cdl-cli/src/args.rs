use std::path::PathBuf;

use clap::Clap;
use semver::{Version, VersionReq};
use uuid::Uuid;

use rpc::schema_registry::types::SchemaType;

/// A tool to interact with services in the Common Data Layer.
#[derive(Clap)]
pub struct Args {
    // The address where the schema registry is hosted.
    #[clap(long)]
    pub registry_addr: String,

    /// What to do for the provided schema.
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Clap)]
pub enum Action {
    /// Work with the schemas in the schema registry.
    Schema {
        #[clap(subcommand)]
        action: SchemaAction,
    },

    /// Work with views of schemas in the schema registry.
    View {
        #[clap(subcommand)]
        action: ViewAction,
    },
}

#[derive(Clap)]
pub enum SchemaAction {
    /// Get the names and ids of all schemas currently stored in the schema
    /// registry, ordered alphabetically by name.
    Names,

    /// Get a schema from the registry and print it as JSON. By default, this
    /// retrieves the latest version, but you can pass a semver range to get
    /// a specific version.
    Definition {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,

        /// An optional version requirement on the schema.
        #[clap(short, long)]
        version: Option<VersionReq>,
    },

    /// Get a schema's metadata from the registry.
    Metadata {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,
    },

    /// List all semantic versions of a schema in the registry.
    Versions {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,
    },

    /// Add a schema to the registry. Its definition is assigned version `1.0.0`.
    Add {
        /// The name of the schema.
        #[clap(short, long)]
        name: String,
        /// The insert_destination of the schema.
        #[clap(short, long, default_value = "")]
        insert_destination: String,
        /// The query address of the schema.
        #[clap(short, long, default_value = "")]
        query_address: String,
        /// The file containing the JSON Schema. If not provided,
        /// the schema definition will be read from stdin.
        #[clap(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
        /// The type of schema. Possible values: DocumentStorage, Timeseries.
        #[clap(short, long = "type", default_value = "DocumentStorage")]
        schema_type: SchemaType,
    },

    /// Add a new version of an existing schema in the registry.
    AddVersion {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,
        /// The new version of the schema. Must be greater than all existing versions.
        #[clap(short, long)]
        version: Version,
        /// The file containing the JSON Schema. If not provided,
        /// the schema definition will be read from stdin.
        #[clap(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },

    /// Update a schema's metadata in the registry. Only the provided fields will be updated.
    Update {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,
        /// The new name of the schema.
        #[clap(short, long)]
        name: Option<String>,
        /// The new insert_destination of the schema.
        #[clap(short, long)]
        insert_destination: Option<String>,
        /// The new query address of the schema.
        #[clap(short, long)]
        query_address: Option<String>,
        /// The new type of the schema. Possible values: DocumentStorage, Timeseries.
        #[clap(short, long = "type")]
        schema_type: Option<SchemaType>,
    },

    /// Validate that a JSON value is valid under the format of the
    /// given schema in the registry.
    Validate {
        /// The id of the schema.
        #[clap(short, long)]
        id: Uuid,
        /// An optional version requirement on the schema. Uses the latest by default.
        #[clap(short, long)]
        version: Option<VersionReq>,
        /// The file containing the JSON value. If not provided,
        /// the value will be read from stdin.
        #[clap(short, long, parse(from_os_str))]
        file: Option<PathBuf>,
    },
}

#[derive(Clap)]
pub enum ViewAction {
    /// Get the names of all views currently set on a schema,
    /// ordered alphabetically.
    Names {
        /// The id of the schema.
        #[clap(short, long)]
        schema_id: Uuid,
    },

    /// Get a view in the registry.
    Get {
        /// The id of the view.
        #[clap(short, long)]
        id: Uuid,
    },

    /// Add a new view to a schema in the registry.
    Add {
        /// The id of the schema.
        #[clap(short, long)]
        schema_id: Uuid,
        /// The name of the view.
        #[clap(short, long)]
        name: String,
        /// Materializer's address
        #[clap(short, long)]
        materializer_address: String,
        /// Materializer's options encoded in JSON
        #[clap(short, long)]
        materializer_options: String,
        /// The file containing the fields definition encoded in JSON.
        /// If not provided, the value will be read from STDIN.
        #[clap(short, long, parse(from_os_str))]
        fields: Option<PathBuf>,
    },

    /// Update an existing view in the registry,
    /// and print the old view. Only the provided properties will be updated.
    Update {
        /// The id of the view.
        #[clap(short, long)]
        id: Uuid,
        /// The new name of the view.
        #[clap(short, long)]
        name: Option<String>,
        /// Materializer's address
        #[clap(short, long)]
        materializer_address: Option<String>,
        /// Whether to update the fields property.
        #[clap(short, long)]
        update_fields: bool,
        /// The optional file containing the fields definition encoded in JSON.
        /// If not provided, the value will be read from STDIN.
        #[clap(short, long, parse(from_os_str))]
        fields: Option<PathBuf>,
        /// Materializer's options encoded in JSON
        #[clap(short, long)]
        materializer_options: Option<String>,
    },
}

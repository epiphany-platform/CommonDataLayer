# Common Data Layer CLI

A tool to manipulate schemas and views in the Common Data Layer.


## Getting Started

To install or run this tool, you will need [Rust](https://rustup.rs/) installed.

To just run it, clone this repo and run `cargo run --bin cdl -- <options>`. 

To install it, clone this repo and run `cargo install --bin cdl`, after which it will
be installed to your `$HOME/.cargo/bin` directory, which if in your `$PATH` will allow
you to run `cdl` directly.

## Usage

For the sake of concision, though you will probably be running `cargo run --bin cdl -- <options>`,
this README will simply describe commands with the shorthand `cdl <options>`.

_Note: This assumes you are running the common data layer locally for now. The ports for_
_schema registry and the storage service are copied from the `docker-compose.yml` file, but_
_if you are using different ports, you should provide those with the `--port` option._

### Schemas

To add a schema to the registry, run `cdl schema add -s <schema_name> -f <schema_file_path>` to read
a schema from a JSON file or `cat <schema_file_path> | cdl schema add -s <schema_name>` to 
read a schema from stdin. Schemas can only be added if they do not exist yet in the schema registry;
the `update` command can be used the same way as `add` to update existing schemas.

To list all schemas alphabetically, run `cdl schema names`.

To get a specific schema, `cdl schema get -s <schema_name>` will print a schema in formatted JSON.

To validate that a JSON value is valid for a given schema, run
`cdl schema validate -s <schema_name> -f <data_path>` to read the data from a JSON file or
`echo <json_string> | cdl schema validate -s <schema_name>` to read the data from stdin.

### Views

To add a view under a schema already defined in the registry, run
`cdl schema views -s <schema_name> add -n <view_name> -v <JMESPath_view>`. Views can only be added
to schemas if they do not already exist on the schema; the `update` command can be used the same way
as `add` to update existing schema views.

_Make sure that views are valid [JMESPath](https://jmespath.org/) expressions._

To list all views of a schema alphabetically, run `cdl schema views -s <schema_name> names`.

To get a specific view on a schema, run `cdl schema views -s <schema_name> get -n <view_name>`.

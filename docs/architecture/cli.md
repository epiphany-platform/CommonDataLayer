# CLI

## Technical Description

The CDL-CLI is the official tool for interacting with the CDL's Schema Registry, used both for viewing and manipulating schemas and their respective data.

For this tool to work, please make sure that the Schema Registry's gRPC server is listening on a public port. Currently, the Schema Registry only exposes a gRPC API, which is faster than a JSON API but less convenient to use. There is some progress with a JSON API for convenience, as well as a TUI (terminal user interface) and a website.

Communication Methods:
- GRPC

## How to guide

For the sake of concision, though you will probably be running `cargo run --bin cdl -- <options>`,
this README will simply describe commands with the shorthand `cdl <options>`.

_Note: This assumes you are running the common data layer locally for now. The ports for_
_schema registry and the storage service are copied from the `docker-compose.yml` file, but_
_if you are using different ports, you should provide those with the `--port` option._

#### Manipulate Views
To add a view under a schema already defined in the registry, run
`cdl schema views -s <schema_name> add -n <view_name> -v <JMESPath_view>`. Views can only be added
to schemas if they do not already exist on the schema; the `update` command can be used the same way
as `add` to update existing schema views.

_Make sure that views are valid [JMESPath](https://jmespath.org/) expressions._

To list all views of a schema alphabetically, run `cdl schema views -s <schema_name> names`.

To get a specific view on a schema, run `cdl schema views -s <schema_name> get -n <view_name>`.

#### Manipulate Schemas

###### Add Schema
`cdl --registry-address "http://localhost:6400 schema <add|get|names|update> --name <schemaname> \
    --query-address <query-service-uri>" \
    --topic <ingest-topic> \
    --file <optional:schema-path>
`

- If `--file` is provided, the specified file must have valid JSON inside.
- If `--file` is missing, the CLI will expect JSON to be piped in over `stdin`.
- A schema containing `true` will accept any valid JSON data.
- New schemas are assigned a random UUID on creation, which will be printed after a successful insert.

###### List Schemas

 To print all existing schema names and their respective ID's:
`cdl --registry-address "http://localhost:6400 schema names`

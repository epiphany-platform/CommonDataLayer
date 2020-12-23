# Command Service

## Technical Description

The CDL-CLI is the official tool for interacting with the CDL's Schema Registry, used both for viewing and manipulating schemas and their respective data.

For this tool to work, please make sure that the Schema Registry's gRPC server is listening on a public port. Currently, the Schema Registry only exposes a gRPC API, which is faster than a JSON API but less convenient to use. There is some progress with a JSON API for convenience, as well as a TUI (terminal user interface) and a website.

Communication Methods:
- GRPC

## How to guide:

#### Manipulate Views
Views are a WIP feature, currently not used widely beside some cases in development.

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

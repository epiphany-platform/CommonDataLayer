# Command Service

## Technical Description

Cdl-cli is currently the only tool able to interact with schema registry. Its purpose is to interact with the schema registry database by setting and viewing its content.

Please mind that this tool is a reference. Schema registry port should be configured open, and every tool that is able to communicate to it should be able to set and get information about schemas from schema registry. Currently only schema regisry only supports GRPC protocol, but there are some progress with both REST and webgui parts, which may be presented in the future as GRPC's alternative.

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

- if `--file` is provided, specific file must have valid json inside.
- If `--file` is missing, the CLI will expect JSON to be piped in over `stdin`.
- A schema containing `true` will accept any valid JSON data.
- New schemas are assigned a random UUID on creation, which will be printed after a successful insert.

###### List Schemas

 To print all existing schema names and their respective ID's:
`cdl --registry-address "http://localhost:6400 schema names`

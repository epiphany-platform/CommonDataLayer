# Schema Registry

A registry of schemas and views for use in the Common Data Layer, written with the [IndraDB][indradb] database.

## Schemas

Schemas are the format in which data is to be sent to the Common Data Layer. Each schema is assigned
a random UUID on creation and initially is created with a name and an initial definition. The name is
not required to be unique among all schemas (as the UUID is the unique identifier of schemas), but is
simply for identifying the schema when searching for schemas. It can be updated at any time.

The definition is a [JSON Schema][json schema] document that describes the expected format of data stored
under the given schema. It is assigned the semantic version 1.0.0, and cannot be updated after creation.
Rather, updates can be made to the definition by inserting another definition with a new semantic version
strictly larger than any other existing version assigned to that schema.

When validating data against a schema, either the latest version of the definition is used, or optionally 
a semantic version range can be provided, and the latest version meeting the range is used.

Schemas can also have multiple views, described below.

An example of a CDL schema might be: 

```json
{
    "id": "<schema UUID>",
    "name": "Vector",
    "definitions": {
        "1.0.0": {
            "x": "number",
            "y": "number",
            "z": "number"
        },
        "1.0.0": {
            "w": "number",
            "x": "number",
            "y": "number",
            "z": "number"
        }
    },
    "views": [
        "<view 1 UUID>",
        "<view 2 UUID>",
        "<view 3 UUID>"
    ]
}
```

## Views

Views describe projections of data defined by a specific schema. As with schemas, each view is assigned
a UUID on creation and initially is created with a name and an initial definition. Like schemas, the
name is a vanity name for searching purposes only and can be updated at any time, as the UUID is the unique
identifier used. On creation, a view is assigned to a schema and cannot be assigned to a different one.

The definition is a [JMESPath][jmespath] expression which describes how to project data defined under the
parent schema into the desired output format. Unlike with schemas, view definitions are editable at any time
and are not versioned, though this feature may be added in the future.

An example of a CDL view might be:

```json
{
    "id": "<view UUID>",
    "name": "two dimensions",
    "definition": "{ x: x, y: y }"
}
```


## Usage

The schema registry provides a gRPC interface, which is easiest to interface with via the [CDL CLI][cdl cli].


[indradb]: https://github.com/indradb/indradb
[sled]: https://sled.rs/
[jmespath]: https://jmespath.org/
[json schema]: https://json-schema.org/
[cdl cli]: ../cdl-cli/

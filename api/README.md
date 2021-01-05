# GraphQL API Server - Overview

server which provides `/graphql` and `/graphiql` routes for CDL management.
It is self-describing, interactive and easy to use way to manage your instance.

# Roadmap

This crate is under heavy work-in-progress. It may change a lot with breaking changes. Use wisely.

## v0.1

* [x] Add schema
* [x] Update schema with new version
* [x] Get all schemas with all views and definitions
* [x] Add view
* [x] Update schema parameters:
  * [x] Name
  * [x] Topic
  * [x] Query address
  * [x] Type
* [x] Update view
* [x] Get single schema
* [x] Get single view
* [x] Documentation comments

## v0.2
* Insert object to CDL
* Retrieve object from CDL
* Use `dataloader` to solve N+1 problem (performance)

# Getting started

You can deploy web api on local computer via `docker-compose`.
For more see [documentation](../deployment/compose/README.md).

You can access interactive graphQL editor at http://localhost:50106/graphiql. It supports auto-completion, has built-in documentation explorer and history. 

Because our schema-registry is automatically initialized with some schemas, you can start making queries right away, like:

``` graphql
{
    schemas {
      id,
      definitions {
        version,
        definition
      },
      views {
        expression
      }
    }
}
```

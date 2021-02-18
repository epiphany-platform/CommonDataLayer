# GraphQL API

server which provides `/graphql` and `/graphiql` routes for CDL management.
It is self-describing, interactive and easy to use way to manage your instance.

# Getting started

You can deploy web api on local computer via `docker-compose`.
For more see [documentation](../deployment/docker-compose.md).

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

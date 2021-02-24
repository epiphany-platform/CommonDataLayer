# GraphQL API

Server which provides `/graphql` and `/graphiql` routes for CDL management.
It is self-describing, interactive and easy to use way to manage your instance.

# Getting started on local machine (via docker-compose)

Check our [guide](../deployment/local/docker-compose.md) to see how to deploy API locally.

You can access interactive graphQL editor at http://localhost:50106/graphiql. It supports auto-completion, has built-in documentation explorer and history. 

Because our schema-registry in docker-compose is automatically initialized with some schemas, you can start making queries right away, like:

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

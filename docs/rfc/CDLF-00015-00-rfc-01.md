# Front Matter

```
Title           : CDL publishing deployment configurations
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Reviewer        : CommonDataLayer
Created         : 2021-05-17
Last updated    : 2021-05-17
Version         : 1.0.0
CDL feature ID  : CDLF-00015-00
```

# Introduction

We need to revisit what we publish in terms of deployment - currently there are multiple configurations present in repositories,
and often we have difficulty keeping them up to date. 
Having multiple deployments increases complexity of every change, as PR must visit much more files, more scenarios have to be tested manually.

On top of deployments comes provisioning database. Currently, we support doing that to Postgres, with Victoria Metrics being automatically in "correct" state,
and Druid is set up by the Client.
Our Postgres has 2 methods of provision present in repository, both of them assuming schema name, so they are not portable in this regard.
Provisioning happens via sqlx and postgres docker volume.

# Glossary
*Migration* - SQL migration, including only CDL internal structure (no schema migrations). Right now it means 0 to x state only.

# Topics
## Deployment

### Currently supported
#### helm
* mandatory (in theory k8s is mandatory, but helm is better replacement)
* we will end up using it for some tests
* we try to avoid using it for development - it's slow to compile crates on k8s, environments can be tricky

#### docker-compose
* optional - initially used for local deployment setup, currently only supplies infra for testing

#### horust
* optional
* linux-only (uses linux api's incompatible with OSX and Windows)
* bare-metal

#### custom bare metal
* optional
* configuration can be propagated to other forms of deployment

### Proposed
We revisit helm and publish a template where you plug in individual toml configurations.
This should reduce helm complexity and ease maintenance.
We will keep cdl-config up to date. A good idea may be using `schemars` crate, or writing our own version with TOML support.
We will provide basic config for `kafka + all repositories` and `rabbit/grpc with postgres`.
With this combination we can manage local development and k8s deployments, and have just one place where we keep configs up to date.
Docker-compose and horust can stay in separate repo, for personal use. We shouldn't require a PR to update those as well 
(not to mention config files can mitigate the need to update).
We shouldn't provide `infra` helm files - just CDL deployment.
Infrastructure `docker-compose` could stay in separate repo for development use.

## Database setup

### Currently
We have docker-compose postgres setup and schema-registry setup in two separate directories.
A client has to download these files and modify them before being able to apply.

### Options
#### Embedding db setup into CDL
Tricky and hard to manage. We have to remember that CDL is not Postgres only. While this may be an option for `edge` and `schema` registries, this is out of question for repositories now.
#### Setting up directory with migrations
We will need to have migrations per `app`, assuming each application can use separate database.
We need to pick a tool to run these migrations.
##### Druid
Probably need to write this tool ourselves. Druid requires conversion from CDL schema to its internal format, and maybe,
assuming `SR-less deployments`, this tool has to be usable without schema whatsoever.
> druid support is currently on hold
##### Influx / VictoriaMetrics
This db seems to not require migrations
##### Postgres
Postgres is very tricky. There's a lot of tools that do what we want in better or worse manner.
* SQLx is rust based option, but it forces us to write scripts with `schema` knowledge - we should avoid it.
* FlyWay is generic tool (could be used with other SQL databases if we ever support them), used by our clients, that can handle multiple schemas etc.
* We can always bash-script these migrations into db.
#### No config files
User would be left alone. We'd still have to provide documentation on how to write these migrations.

---
I'd vote for having migrations in folder structure `deployment/migrations/{app}` for `edge-registry` and `schema-registry` and
`deployment/migrations/{db}-repository` for repositories. We don't mention any tool or how to use them. We will manage something internally.

This also means that no `druid-conversion-tool` now, only `../migrations/postgres-repository` will exist.

As for `migrations` name. Now they will be only single script per tool, but we have to future-proof ourselves to a moment when a repo will require
change of db-schema.

# Decisions

> Actions are from meeting about deployment that happened on 24.06.2021.

## Helm

Some additional work has to be done with helm. It will stay within main repository, however we need to apply changes to charts.
Issue https://github.com/epiphany-platform/CommonDataLayer/issues/583 was created to accommodate work that needs to be done.
We are going to switch helm to support configuration tomls.
Caveats are:
* configuration must be dynamic and partially based on values.
* we have to keep in mind that host addresses depend on `deployment` name.
* user should be able to configure communication method or repository kind they're deploying.

## Docker-Compose

docker-compose will be removed from main repository. We'll place development composes of databases/kafka/amqp/etc. in https://github.com/epiphany-platform/CommonDataLayer-deployment.
We'll depend one repository on another, so that postgres database is setup via scripts from main CDL repo.

We may verify whether using submodules or depending on repository being at specific path is better.

## Horust/bare-metal.

Horust and development config files may stay where they are. Both methods are there for fast and easy, non-production setup. They don't introduce
lots of extra clutter.

## Postgres

We'll split current db init scripts into separate directories. At moment of writing this, that will be `document-repository`, `edge-registry` and `schema-registry`.
We won't provide any way of running these on production databases. Helm will adjust to linking from multiple folders.

Proposed path: `{root}/deployment/db-setup/postgres/{app-name}`.


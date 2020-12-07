# Setting UP local environment

## Preamble

Contents of this folder aren't meant for use on production and they may be lagging behind our k8s deployment. 
Sole purpose of this directory is to prepare exemplary development environment, from which anyone can startup their development on 
`common data layer` without Kubernetes knowledge. Contents of docker-compose may not contain all applications, so be aware of that. You may alter it
on your local machine to your needs.

For k8s deployment, please refer to our [documentation](../../docs/k8s_local_deployment.md). 

## Requirements
* docker
* docker-compose
* rust (optionally)

## Volume

The directory `./docker-volume` is used as a volume. Please note it is not fully `.gitignore`d because we rely on `init.sql` for postgres, both in docker-compose and in Github Actions. DO NOT REMOVE IT UNDER ANY CIRCUMSTANCES.

## Deployment
You must first add environment variables:

`DOCKER_BUILDKIT=1`  
`COMPOSE_DOCKER_CLI_BUILD=1`

Due to services needing additional startup time, we advise to let docker setup infrastructure first, and deploy CDL after. So...

`docker-compose up -d kafka zoo postgres`

After it had some time to setup, you can proceed with rest of the environment:

`docker-compose up -d`

## Entry points in system
### Kafka

You can write to kafka on `localhost:9092`.
By default there is no replication on *schema_registry* and postgres *command_service* input channel is `cdl.document.input`.

Errors are written to `cdl.reports`.

### PostgreSQL

To access postgres you must have some postgresql client installed.

For command line it's best to refer to your OS package manager (`homebrew` on OSX, `apt-get` on Ubuntu, `choco` on Windows).

`psql -U postgres --password -h localhost`
the password is `1234`

### Schema registry
Schema registry can be either accessed via [gRPC](schema-registry/proto/registry.proto), or via `cdl-cli`. Using `cdl-cli` will require presence of rust compiler on your local machine.
Tips on how to install rust are available on [rustup website](https://rustup.rs/).

From main directory of this project you can run `cdl-cli` via:

`cargo run -p cdl-cli -- --help`

Registry address is `http://localhost:50101`.

eg.

* Adding new schema:
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema add --name default-document`

* Setting schema topic (in order for this schema to be routed to `command-service` topic must be `cdl.document.input`)
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema set-topic --id 0a626bba-15ff-11eb-8004-000000000000 --topic "cdl.document.input"`

* Getting all schemas
> `cargo run -p cdl-cli -- --registry-addr "http://localhost:50101" schema names`

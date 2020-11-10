# Testing

This directory contains integration tests for *CDL* components.

Making sure they pass is vital to longevity of our application.

## Prerequisites

### Python
Tests run on python `3`. They are tested on CI using python `3.8`.

### Pip
We require `pip` to be present in `$PATH`. Some python installations, especially when you are keeping both `python2` and `python3`
on your machine, tend to reserve `pip` as `python2` package manager. Before using, please ensure whetreh that's the case on your system.
If it is, please adjust executable name to `pip3` in following commands, or symlink `pip3` to `pip`

### Rust & Cargo
Tests don't compile *CDL*. You have to do it yourself, and either install *CDL* binaries into your `$PATH` or point tests to binaries via env variables.

Supported env variables are:

| application | env |
|---|---|
| db-shrinker-postgres | DB_SHRINKER_POSTGRES_EXE |
| command-service | COMMAND_SERVICE_EXE |

### Accompanying services
In some cases our applications require database, message queue or some other service in order to function. These can be started in two ways.
In order to setup infrastructure stack on kubernetes, please refer to [helm deploy](../docs/K8s-Local-Deployment.md).
In order to setup infrastructure stack locally via docker-compose, please refer to [docker-compose examples](../examples/deploy/SETUP.md).

Access to such set up services is managed via env variables:

| service | env | example |
|---|---|---|
|PostgreSQL database | POSTGRES_CONNECTION_URL | postgresql://postgres:1234@localhost:5432/postgres |
| Kafka broker | KAFKA_BROKERS | localhost:9092 |

## Running
First we need to initialize environment from `requirements.txt` located at the top of project directory.
We do that by running `pip install -r ../requirements.txt`.

`pytest` discovers tests and runs them all.

## Structure
 
### ./data
Folder with test data, divided into directories per application

### application folders
eg. db_shrinker_postgres. Named after application they test.

# Setting UP local environment

## Preamble

> Intended way of deploying CDL is through helm files.

Contents of this folder aren't meant for use on production and they may be lagging behind our k8s deployment. 
Sole purpose of this directory is to prepare exemplary development environment, from which anyone can startup their development on 
`common data layer` without need of Kubernetes knowledge. Contents of docker-compose may not contain all applications, so be aware of that. You may alter them
on your local machine to your needs.

For k8s deployment, please refer to our [documentation](../../docs/k8s_local_deployment.md). 

## Requirements
* docker
* docker-compose
* rust (optionally)

## Deployment
You must first add environment variables:

`DOCKER_BUILDKIT=1`  
`COMPOSE_DOCKER_CLI_BUILD=1`

Environment with infrastructure alone is started via:

`docker-compose up -d`

Adding new components happens through invoking `docker-compose` files containing parts of `CDL`.
First, you must spin up `base` services, eg. for Kafka use:

`docker-compose -f docker-compose.cdl.kafka.base.yml up -d`

Then you can add repositories:

`docker-compose -f docker-compose.cdl.kafka.postgres.yml up -d`

Make sure that repository you've chosen matches `base`'s communication protocol.

### ./setup
Directory contains setup scripts for `postgres`, `schema_registry`, etc..

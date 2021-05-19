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

`./setup.sh` is responsible for modifying/selecting deployments for you.
It accepts command line args:
`./setup.sh COMMUNICATION_METHOD REPOSITORY_KIND`

**COMMUNICATION_METHOD**
* kafka (materialization is only included via kafka atm)
* amqp
* grpc

**REPOSITORY_KIND**
* postgres
* victoria_metrics
* druid (supports only kafka)

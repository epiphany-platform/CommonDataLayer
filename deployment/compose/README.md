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

### Exposed ports
Dependencies
| Port exposed | Service name                    |
|--------------|---------------------------------|
| 9092         | kafka                           |
| 9093         | kafka                           |
| 5432         | postgres                        |
| 8428         | victoria metrics                |
| 5672         | rabbitmq                        |
| 15672        | rabbitmq                        |
| 8081         | druid coordinator               |
| 8082         | druid broker                    |
| 8083         | druid historical                |
| 8091         | druid middlemanager             |
| 8888         | druid router                    |
| 6831/udp     | jaeger                          |
| 6832/udp     | jaeger                          |
| 16686        | jaeger                          |
| 14268        | jaeger                          |

Services
| Port exposed | Service name                    |
|--------------|---------------------------------|
| 50101        | schema registry                 |
| 50102        | data router (gRPC only)         |
| 50103        | query router                    |
| 50104        | **FREE**                        |
| 50105        | **FREE**                        |
| 50106        | web api                         |
| 50107        | object builder                  |
| 50108        | materializer ondemand           |
| 50109        | **FREE**                        |
| 50110        | edge registry                   |
| 50201        | query service, query service ts |
| 50202        | command service (gRPC only)     |
| 50203        | materializer general            |

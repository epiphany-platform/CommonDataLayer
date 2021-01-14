# Common Data Layer

A collection of services that make up the Common Data Layer, implemented in the [Rust][rust] language.


## What is the Common Data Layer?

The Common Data Layer (CDL) is a data storage service. It is designed with performance, versatility,
scalability, and ease-of-modification as key tenets of its design, among others.



## How does it work?

Data intake is all performed over Message Queue and via the Data Router. Message Queue (MQ) is an abstract entity and the CDL currently supports [kafka][kafka] and [RabbitMQ][rmq]. CDL listens over a single topic queue for messages keyed on strings, each providing a schema ID. The schema ID is used to load the appropriate topic (stored per-schema in the schema registry), which is used to route the message along to the correct repository.

_Note: the crate for the [schema registry][schema registry] has more information on schemas and views._

For each repository, a command service is listening to its specific MQ topic for incoming messages. Each message is stored according to the repository's format. Though most of our command service implementations use append-only storage with each value under a key being assigned a version, it is not required by user-implemented command services.

The query router is used to direct requests for data to the appropriate repository. Each repository also has a query service listening for gRPC requests for data. These query services are used for direct queries of data from the repositories. As repositories are meant to be easily introduced to an already running CDL, but the topic per repository can't be used to make a gRPC request, each schema also stores the dynamic address of the query service it belongs to.


## Structure

This project is structured as a collection of Rust crates:

Crate Name              | Purpose
------------------------|--------
data-router             | Route incoming data from and through MQ for consumption by the specific Command Service
schema-registry         | Manage user-defined schemas that define the format of incoming values and their respective topics
query-service           | Wrap each individual repository for retrieval of data
command-service         | Intake data from a MQ and storage, in specific repository
leader-elector          | Elect master nodes in replicated services (_only for the Schema Repository, currently_)
db-shrinker-storage     | A service to remove older data from storage
query-router            | Route incoming requests to query service given a schema and object id
utils                   | A collection of utilities used throughout the Common Data Layer
cdl-cli                 | Provide a command-line interface for managing schemas in the schema registry and storing and retrieving data

In addition to the above crates, there are some other useful directories:

Directory       | Purpose
----------------|--------
deploy/helm     | helm charts for remote deployment
deploy/compose  | sample deployment guide for docker
benchmarking    | scripts and scaffolding data for benchmarking
tests           | component tests
examples        | examplary client of cdl
docs            | cdl documentation

## Getting Started

See the [getting_started.md][Getting Started] to see how to use this service.


[rust]: https://www.rust-lang.org
[sled]: https://github.com/spacejam/sled
[kafka]: https://kafka.apache.org/
[rmq]: https://www.rabbitmq.com/
[schema registry]: ./schema-registry/
[Getting Started]: docs/getting_started.md

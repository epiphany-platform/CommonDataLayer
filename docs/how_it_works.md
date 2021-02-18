# How does it work

Data intake is all performed over Message Queue and via the Data Router. Message Queue (MQ) is an abstract entity and the CDL currently supports [kafka][kafka] and [RabbitMQ][rmq]. CDL listens over a single topic queue for messages keyed on strings, each providing a schema ID. The schema ID is used to load the appropriate topic (stored per-schema in the schema registry), which is used to route the message along to the correct repository.

For each repository, a command service is listening to its specific MQ topic for incoming messages. Each message is stored according to the repository's format. Though most of our command service implementations use append-only storage with each value under a key being assigned a version, it is not required by user-implemented command services.

The query router is used to direct requests for data to the appropriate repository. Each repository also has a query service listening for gRPC requests for data. These query services are used for direct queries of data from the repositories. As repositories are meant to be easily introduced to an already running CDL, but the topic per repository can't be used to make a gRPC request, each schema also stores the dynamic address of the query service it belongs to.

[kafka]: https://kafka.apache.org/
[rmq]: https://www.rabbitmq.com/

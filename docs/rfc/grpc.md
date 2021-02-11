# Front Matter

```
Title: GRPC communication method as an alternative to Kafka and RabbitMQ
Author: Wojciech Polak
Team: CDL
Reviewer: CDLTeam
Created on: 10/02/2021
Last updated: 10/02/2021
Tracking issue:
```

# Introduction
## Summary

We should add GRPC communication in all components that are using right now Kafka or `RMQ`. We should introduce a standard interface and separate whatever we are doing with input messages from the transportation layer.

## Glossary

`RMQ` names any `AMQP` server - most common is`RabbitMq` \
`MQ` - Message Queue - Kafka or `RMQ` \
`CS` - command service

## Background

Right now, most of our communication in CDL ingestion is handled by either Kafka or `RMQ`. While this is acceptable for some clients, there is a case where CDL should not communicate via message queue at all. Therefore we need a replacement protocol, and we could use GRPC for that purpose.

What is more - right now, we have a partially-baked solution in `CS` - this service accepts either `MQ` or `GRPC` as a communication method; however, `DR` can only produce messages to Kafka or `RMQ`. Furthermore, this solution has mixed business logic with the transportation layer, which causes some unnecessary repetitions in the codebase. It makes it harder to maintain than it should be.

## Goals and Requirements
* Accept `GRPC port` env variable (or command-line argument).
* If GRPC has been chosen over `MQ` - start endpoint.
* Each service should share code between the `MQ` handler and `GRPC` handler.
* Code handling message should not depend directly on any communication method. It should be under an abstraction.

# Solutions
## Existing solution

We are currently using `GRPC` only for querying data and communication between CLI/GUI/API and schema registry.

`GRPC` support in the ingestion part of CDL is not finished.

A client cannot use CDL without `MQ`.

Additionally, right now, CDL uses the same topic for both error notifications and report notifications.
* Error notifications inform the client about errors, for example, because of a corrupted message - it has the same purpose as good logging and monitoring. In the GRPC world, we can also return information about the error in the response.
* Reports inform clients about the resolution on `object` level - if data sent to `MQ` has been processed by CDL and stored in DB or rejected.

CDL needs notifications in the async world because there is no way to inform the client about corruption/resolution directly; however, this is not necessary in the GRPC world (at least not the error notification part).

What is worth mentioning - this RFC requires finding some middle ground between sync (`GRPC`) and async (`MQ`) world. 

By saying `GRPC` in sync, we mean - it requires that each request returns a response, while `MQ` has more *fire and forget* behavior. In both scenarios, rust implementation is using tokio and async-await features.

These features, unfortunately, are not common.
`MQ` is heavily based on the `Stream` trait while `GRPC` uses `async-trait`.

Unfortunately, using `Stream` in Kafka requires box leaking, which might be dangerous when left alone. The message is also wrapped into the `Box` to allow dynamic dispatch (to acknowledge either Kafka message or `RMQ` message).

## Proposed solution

### Async trait
As previously mentioned, the transportation layer should be invisible to the user. To do so, I'd like to introduce a new async trait:
```rust
trait ConsumerHandler {
    async fn handle(&self, msg: &dyn Message) -> Result<()>;
}
```

Each service would implement that handler trait to receive messages from `MQ`/`GRPC`.

We ditch here the `Stream` trait - unfortunately, while it is a powerful tool, we cannot use it for GRPC. It also means we remove dangerous box leaking.

Second of all message is no longer wrapped in `Box` - instead, the user receives only **reference** to the dynamic object.

Lastly - handler returns `anyhow::Result` - so transportation layer based on that can:
* `GRPC` - return response either OK/Internal Server Error/Bad Request (TBD how to distinguish between last two)
* `MQ` - use acknowledge, negative acknowledge (in `RMQ`) or only doing nothing and not responding at all to the message broker.

### Internal implementation
Internal implementation is quite simple. We keep `enum Consumer`, which accepts in its constructor our instance of `ConsumerHandler` along with configuration parameters (URL address to Kafka broker etc.).

Inside of method `async fn run(self)` we match consumer variant and either run simplest possible `while let Some()` loop for `MQ`, or initiate `GRPC` server.

Per each received message (either from `MQ` or in `GRPC` server implementation), we can call `consumer_handler.handle(&msg)` and wait for the response. Simple as that.

It means that in the transportation layer for `MQ` all messages are processed in order (per partition in Kafka world). If one needs more performance - she can always use `tokio::spawn` **inside** of consumer handler and return the handle eagerly.

### GRPC protocol schema

To unify all internal communication in CDL ingestion, we need to use a common, shared GRPC protocol.
```proto
syntax = "proto2";

package generic_rpc;

service GenericRPC {
    rpc Push(Message) returns (Empty);
}

message Message {
  required string key = 1;
  required bytes payload = 2;
}

message Empty {}
```
Thanks to that, we can imitate a message just like the one received from `MQ`.

### Notifications & Error reporting

No client requires error reporting, and we are already sending logs. Therefore it is not needed and can be removed.

Notifications are a bit more complicated. These are `CS` specific, and therefore, cannot be part of the transportation layer (transparent to the user code).

We need to introduce `ReportSender` for GRPC - One cannot send the report to any `MQ`. One suggested way is to send the callback to some specified endpoint by sending a `POST` request. Another is to use elastic search or Postgres.
i
This issue is open for further discussion.

# Test Plan

TBD.

There should be at least one end-to-end test checking if the whole pipeline works in an `MQ`-less environment.

# Futher considerations
## Impact on other teams

Teams that are using `MQ` won't feel any difference. This refactor would allow other clients to use CDL.

## Security

No security risk.

# Tasks and timeline

TBD

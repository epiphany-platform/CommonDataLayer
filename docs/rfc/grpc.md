# Front Matter

```
Title: Alternative communication method to Kafka and RabbitMQ
Author: Wojciech Polak
Team: CDL
Reviewer: CDLTeam
Created on: 11/02/2021
Last updated: 11/02/2021
Tracking issue: https://github.com/epiphany-platform/CommonDataLayer/issues/68
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

Unfortunately, because Kafka uses borrowed messages it requires box leaking, which might be dangerous when left alone. The message is also wrapped into the `Box` to allow dynamic dispatch (to acknowledge either Kafka message or `RMQ` message).

## Proposed solution

### Async trait
As previously mentioned, the transportation layer should be invisible to the user. To do so, I'd like to introduce a new async trait:
```rust
trait ConsumerHandler {
    async fn handle(&self, msg: &dyn Message) -> Result<()>;
}
```

Each service would implement that handler trait to receive messages from `MQ`/`GRPC`.

First of all, while we switch from `Stream` trait to `async trait`, we cannot simply remove `Box::leak`. We could do it when we limit the code to the ordered single-threaded solution, however that would create a hughe performance hit.

Instead, we still need to rely on leaking because `tokio::spawn` (called inside of the transportation layer) requires `'static` lifetime. In the future we might be able to either ditch borrowed message and replace it with owned, or use proposed structured concurrency which would enable to spawn task with some non static lifetime (because we could guarantee that all tasks should finish before we drop the consumer).

Second of all message is no longer wrapped in `Box` - instead, the user receives only **reference** to the dynamic object.

Lastly - handler returns `anyhow::Result` - so transportation layer based on that can:
* `GRPC` - return response either OK/Internal Server Error/Bad Request (TBD how to distinguish between last two)
* `MQ` - use acknowledge, negative acknowledge (in `RMQ`) or only doing nothing and not responding at all to the message broker.

### Internal implementation
Internal implementation is quite simple. We keep `enum Consumer`, which accepts in its constructor our instance of `ConsumerHandler` along with configuration parameters (URL address to Kafka broker etc.).

Inside of method `async fn run(self)` we match consumer variant and either run simplest possible `while let Some()` loop for `MQ`, or initiate `GRPC` server.

Per each received message (either from `MQ` or in `GRPC` server implementation), we can call `consumer_handler.handle(&msg)` and wait for the response. Simple as that.
To enable better performance we would call this handler inside of `tokio::spawn`. What is worth mentioning, `MQ` acknowledges should be sent inside of the spawned task.

`GRPC` is unordered by design, if client needs an ordering, it needs to send one request at the time - it is it's responsibility, not CDL.

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

We need to introduce `ReportSender` for GRPC.
An abstracted producer which sends reports to given sink. 

One cannot send the report to any `MQ`. One suggested way is to send the callback to some specified endpoint by sending a `POST` request. Another is to use elastic search or Postgres.

This issue is open for further discussion.

## Alternative solutions
### REST
Instead of using GRPC we could use simple HTTP REST requests.

#### Advantages
* Simpler to implement
* Do not need protocol schema
* Based on HTTP - usable with service mesh
* No mixed protobuf with pure JSON in payload - simpler to deserialize

#### Disadvantages
* Requires extra effort to switch from GRPC
* Does not solve client code generation like GRPC

### Custom TCP
Instead of relying on existing protocol, we could also create custom one based on TCP.

#### Advantages
* Full controll on design
* Probably fastest method (if done right)

#### Disadvantages
* Requires a lot of effort with designing, testing and benchmarking,
* Needs extra careful touch in areas regarding timeout connections, closed sockets etc.
* Requires writing custom client libraries in popular languages: Java, Python, C#, JavaScript

### ZeroMQ
While we shouldn't replace Kafka nor RabbitMQ with zeroMQ, we can consider it for ReqResp.

#### Advantages
* Fast, marketed as "zero-abstraction"
* Similar interface to other `MQ` - it could ease creating an abstraction

#### Disadvantages
* Rust client library lacks good example how to use it in `async-await` environment,
* There are pinpointed problems with ZMQ listed by one of Rust client maintainers: https://github.com/jean-airoldie/libzmq-rs/issues/125#issuecomment-570551319

## Conclusion

GRPC seems to be easiest way to implement `MQ`-independence, however later we should re-evaluate REST and probably switch to it.
What is worth noting, after we switch CDL to one, abstracted and unified transportation layer, further change from GRPC to REST should be relatively easier.

# Test Plan

There should be at least one end-to-end test checking if the whole pipeline works in an `MQ`-less environment.
We should run exactly same tests as we have now but with different env variables. All tests designed for `MQ` environment should pass in `GRPC` environment.

# Futher considerations

## Schema Registry replication
Replication featutre for Schema Registry needs major refactor (and probably replacement), therefore it is out of the scope of this change. Replication for `GRPC` should be deactivated.

## Impact on other teams

Teams that are using `MQ` won't feel any difference. This refactor would allow other clients to use CDL.

## Security

No security risk.

# Tasks and timeline

TBD

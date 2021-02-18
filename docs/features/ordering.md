# Message ordering

## Index
- [What is message ordering](#what_is)
- [Why does it matter](#why)
    - [Football/soccer game](#soccer)
    - [Network traffic monitor](#trafic_monitor)
    - [Pros and cons](#pros)
- [Message ordering in CDL](#message_ordering_in_cdl)
- [How to use it](#how_to_use)
    - [Overall](#overall)
    - [Communication through Apache Kafka](#kafka)
    - [Communication through RabbitMQ](#rabbit)
    - [Communication through RPC](#rpc)
  

## <a name="what_is"></a>What is message ordering 
Message ordering is a guarantee that certain messages will be processed by CDL in the same order there were sent to the system. Proper ordering matter mostly to user applications which base their business logic on real-time data where no approximation is allowed. 

## <a name="why"></a>Why does it matter 
Let's consider two use cases which will show us why order of received messages might/might not matter: 

### <a name="soccer"></a>Football/soccer game 
We develop an application which decides strategy for playing football matches. Based on actual score we decide if our team should play more offensively, defensively or utilize balanced play style. 

Incoming events: 
- (1) Match start 
- (2) We scored a goal 
- (3) Enemy scored a goal 
- (4) Match end 

Linearizable system (system which can establish exact order for each message), will see events in the order they were produced. Our application will start game with balanced play style, go defense after 2nd event is received (we scored a goal) and change it back to balanced once enemy hits us back. 

Without proper message ordering same situation can be processed (seen by clients) differently. If second message got delayed for some reason and came out of order, we could play with completely different strategy. We would start game in balanced formation, then the 3rd message will show up (enemy scored a goal), so we'll think we're losing the game and start playing offensively. After some time, 2nd message will finally show up and we'll end game in balanced formation. 

From user perspective it would seem that our program is broken because we played more offensively once we were one point ahead of the enemy, which could cause us to lose our advantage. 

It's worth noting that if we play other sport discipline e.g., a basketball or volleyball making decisions with few delayed messages can be considered valid behavior - there are more data points, so it would be fine to use approximated score (difference of one or two points doesn't change the general strategy because there are much more points in general). In such case we don't make our decision based on loosing single point but make it if we're losing by a few points. Loosing single point (single message) has a small impact on our decision-making process. 

### <a name="trafic_monitor"></a>Network traffic monitor 
We develop an application which measure network traffic. It can show different statistics per chosen period. 

We're looking for statistics, so it's often fine to approximate the data. We mostly do our job on many data points at once (time period) so even skipping some of them would mostly be fine. What is also important is the fact that we're not processing a real-time data - we're showing data from some time ago (a month, an hour etc.), so even if messages come in the wrong order, they will be available in the system once we query them. 

In this case message ordering is not that important. We're using historical data which will be correct regardless of message ordering.  

### <a name="pros"></a>Pros 
- without message ordering system might see a state of things that have never happened (not just delayed state) 

### <a name="cons"></a>Cons 
- fully linearizable systems can be really slow - inerrability drastically limits system ability to process data in parallel (and scale horizontally) 

## <a name="message_ordering_in_cdl"></a>Message ordering in CDL 
CDL supports three message ordering strategies: 
- Fully ordered messages(linearizable) 
- Message ordering defined by causality 
- Unordered messages (no message ordering guarantees) 

Message ordering defined by causality is a middle ground between two opposite strategies. It allows you to keep message ordering for some of the messages without performance costs of full linearization. E.g., in our first example we could say that order of messages regarding same game is important, but we don't care about order of two messages related to different sport events. 

## <a name="how_to_use"></a>How to use it 
### <a name="overall"></a>Overall 
In CDL message ordering guarantees are defined on per message level. In CDL data ingestion message format there is an optional field called `order_group_id`. This field should contain user generated UUID with ordering info. If you set this value CDL guarantees that messages with the same `order_group_id` will be processed in the order they were send to CDL. Otherwise, if field is left empty, no ordering guarantees are met for this message. This behavior allows us to support causality ordering (multiple `order_group_id` values), linearizability (same `order_group_id` for each message) or to skip ordering guarantees at all(`ordering_group_id` not provided). 

### <a name="kafka"></a>Communication through Apache Kafka 
If you’re using Kafka as a message bus following requirements needs to be met for message ordering to work correctly: 
- Kafka partitioning should be based on message key 
- Message keys of data coming to CDL should be set to `order_group_id` or left empty if message order is not important
- Scaling data router and command service is possible up to number of Kafka partitions. If you need more service instances you must have enough Kafka partitions to feed them messages. 

### <a name="rabbit"></a>Communication through RabbitMQ 
If you’re using RabbitMQ as a message bus following requirements needs to be met for message ordering to work correctly: 
- You must create proper exchange-queue bindings in RabbitMQ 
    - create 2+ queues - one for unordered messages, one or more for ordered ones; number of queues is the limit of horizontal instance scaling (exception - unordered messages) 
    - create one exchange which name will be saved in schema registry
    - create binding between exchange and queues in a way that messages with `unordered` message key will go to queues with unordered data, other keys will be split between other queues 
- Configure command service instances: 
    - Unordered message queue can be passed to each command service instance 
    - Ordered message queue can be passed to single command service (exclusive consumer) 
- Message keys of data coming to CDL should be set to `order_group_id` or left empty if message order is not important

Unfortunately, that means that scaling command services can be done only manually (automatic scaling may be implemented by [#185](https://github.com/epiphany-platform/CommonDataLayer/issues/185)), instances which process only unordered messages can be scaled automatically. 

### <a name="rpc"></a>Communication through RPC(WIP) 
In case of communication through RPC message ordering is guaranteed by request/response pattern and its client responsibility to decide if messages can be sent (and processed) in parallel. `order_group_id` field is ignored. 
 

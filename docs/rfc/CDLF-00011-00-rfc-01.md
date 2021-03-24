# Front Matter

```
Title: Materialization
Author: Sam Mohr
Team: CDL
Reviewers: CDL Team
Created on: February 7th, 2021
Last updated: February 11th, 2021
Tracking Issue: https://github.com/epiphany-platform/CommonDataLayer/issues/227
```


# Introduction

Demand has been expressed for Materialization by multiple clients.


## Summary

Materialization is the projection of value(s) into a different format.

Materialization in the Common Data Layer will need to provide some means
to project data (either a single value or a combination of values) in a
user-defined manner.


## Context

### Origin

Materialization was suggested as a potential feature in the CDL a while ago
as a means for teams using the CDL for storage to make complicated queries
quicker to read by simply precalculating the results and then caching them.
It was recently brought up again as potential client teams saw the usefulness
in materialization and requested it be implemented for their use.

### Previous Efforts

The CDL Lite, a small, monolithic version of the CDL created during the
prototyping phase of the CDL for testing performance, has a version of
materialization implemented. The solution involves adding two extra types
to each Schema: Views and Relations. 

Schemas can have zero to many views, and views are each defined as a format
to map a value belonging to a schema into using [JMESPath][jmespath]. Any schema
can point to any other schema (like a directed edge in a graph) with a relation,
which defines a parent-child relationship between schemas. For safety, there is
a recursion limit when loading child data to allow users to configure cyclical
relationships but preventing memory overloads.

The combination of views and relations allows for deterministic projection of any
value under a schema into a user-defined format, even allowing for the inclusion
of pluralities of data belonging to other schemas. However, this solution was only
implemented in the CDL Lite because the CDL Lite runs on a single binary and a
single database, and has no need to use the network to retrieve data from other
repositories. If a schema's relation points to data in other repositories, the full
version of the CDL would need to make a network request for data in any different
repository at least once and probably more, which incurs a very high networking cost.
This would need to be mitigated if we consider this solution for the full CDL.

### How It Affects Other Components

The CDL was designed to be agnostic of the format of data stored in it, as well
as where it is stored. To that end, all possible repositories simply implement a
common interface for accepting arbitrary data for storage, and those repositories
don't communicate with each other, only ingesting from the common Data Router and
reporting insertions of data to a common Kafka topic.

However, data can be directly inserted into any repository through its respective
Command Service, and if a user uses their own Command Service that they wrote
themselves, then there is no constraint currently requiring them to properly report
insertion events over Kafka.

If some materialization is recalculated based on when data updates, we don't have
a consistent means (currently) of determining when data was successfully inserted
into an arbitrary repository, and will need to find one, or only support periodic
recalculations if any.


## Technical Requirements

We will need to provide projection either for single values or multiple values, even
distributed across multiple repositories. The projection will need to be available
both on-demand and recalculated automatically when values are updated in storage.
For at least projections recalculated on change, they will need to be cached in
ElasticSearch, and potentially other user-defined caches in the future.

## Out of Scope

At least for now, partial updates of cached data are out of scope for this feature.
They are extremely complex in their interaction with the proposed approaches in this RFC,
and would take far too long to implement in the same time span as the rest of the work
entailed by this feature.

## Future Goals

Though generically implementing storage in caches like ElasticSearch in this story
will simplify later efforts to support other caches (e.g. MySQL, Redis, etc.), it
is not a requirement. However, at some point, it would be useful to provide these 
other caches for users and allow them to configure which caches to either store all
data in, or the data for specific projections.


# Solutions

As materialization will likely comprise multiple components all forming a solution,
this RFC will discuss the options for each component, which are:

- Method of Materialization
- On-Demand Materialization (calculation of projections per request)
- Cached Materialization (calculation of projections without prompting)
- Caching of Materialization (storage of projections in a cache, namely ElasticSearch for now)
- Configuration of Materialization


## Method of Materialization

### Method 1: Adaptation of CDL Lite Approach

The CDL Lite approach ([seen above](#previous-efforts)) describes adding Views and Relations
to schemas such that any value under a schema can deterministically be projected (including
child values belonging to child schemas based on the parent schema's relations). For example,
given a schema `Work Order` that has a child schema `Asset`, the definition of a view `Cost`
written as the [JMESPath][jmespath] `{ cost: cost + sum(assets[].cost) }` for `Work Order`s,
the `Cost` view could be deterministically calculated for each `Work Order`.

Views would have the fields `name` and `jmespath`, and schemas would have zero to many views.
Relations would have the fields `optional`, `terminal`, and `path`. `optional` determines whether
an error should be thrown if no children are present. `terminal` determines what type of ID's are
found at the end of the `path` for this relation (a single ID, a list of ID's, or a map of ID's).
The `path` is the path to the ID's of the children from the root of a child value, e.g. a path
of `[ "birthday", "id" ]` in an object

```json
{
  "name": "John Doe",
  "birthday": {
    "month": "February",
    "day": 12,
    "year": 1975,
    "id": "valid-uuid"
  }
}
```

will render "valid-uuid".

To calculate the projection for a view, first get all of the children of the given value based
on the child relations of its schema (and their respective children recursively) replace the ID's
at the end of each `path` with the value under the respective ID, and then calculate the view of
the data with all child data incorporated.

#### Pros

This solution offers highly configurable output, even beyond the combination of arbitrary
values, this allows for complex operations on the data using anything that JMESPath provides.
Also, rather than having to specify a view for every specific value, any value under a schema
can be automatically materialized.

#### Cons

Unfortunately, as mentioned with this solution's place in the CDL Lite, the number of network
requests in this solution is O(m + n), where m is the number of schemas and n is the number of
data values requested. So long as users can use these features freely, there is potential for
serious performance loss. The best option is to keep these complicated features, but to inform
users that using complex views and relations will cause slow performance. In addition, partial
updates will be extremely difficult to implement later.

### Method 2: Simple Joins Only

With the versatility of JMESPath and it's options for manipulating data, users are able to perform
an extremely wide number of complex calculations and aggregations of the data stored in the CDL.
This makes it more difficult for us to determine what users have done, and by extension, to know
accurately what data under a schema requires updating, forcing an update of every value under a
schema when any dependency changes.

Most of the behaviors we are providing to users for the combination and aggregation of data are
available in the cache services, but would be slower and defeat the point of precalculating and
caching their desired format. However, from when materialization was first proposed, the wants
of users were very simple: just the ability to "join" data.

To simplify things, we can constrain the possible operations in JMESPath by either removing the
included functions like addition and division, or by forking JMESPath to define our own
highly-simplified version of the format (both to constrain users and our development time).

#### Pros

Either way, simply providing a way to reshape data while still providing all of the other features
in the same manner will satisfy most of our users without giving them a potential footgun that
will need to be strongly documented.

#### Cons

This approach would take more research, especially if we decide to fork JMESPath instead of
constraining the existing implementation. Also, for those users that would prefer to do all of the
work in one spot, they will be left needing to do work in two places: in the CDL and then in their
chosen cache (e.g. ElasticSearch).


## The Materialization Service(s)

The materialization service(s) for eager and automatic projection will each be responsible for
collecting data from various repositories, manipulating it, and either sending it somewhere. In
this shared behavior leaves two solutions: move the common behavior to a library and render as 
two applications, or write a single common application that performs both tasks internally and
has one input port for materialization requests as well as a "cron" job for materialization to
be sent to a cache such as ElasticSearch.

### Method 1: A Library Used by 2 Services

A library is the de facto way to move common behavior from multiple applications to one place.
Thus, in spite of the cost of adding yet another crate to our compilation process, the code cleanup
and resultant simplification of the 2 services would be great for long-term maintainability. Also,
in deployment of multiple instances of each application, resource usage would be more correctly
distributed where needed when the services run in different processes, especially if there is a
significant difference in the number of requests to one service or another.

### Method 2: A Single, Common Service

On the other hand, the library holding the common materialization logic would not be useful
elsewhere in our application, and may not be worth the effort to separate from our services. Though
on the surface, there is benefit in code structure and proper resource distribution with the 2
service-approach, code structure can be achieved with good use of modules, and `tokio` will properly
handle the distribution of resource if the application properly leverages async behavior, which the
CDL already does everywhere else.


## Caching of Materialized Data

The current expected behavior is to store all data in ElasticSearch, but in keeping with the CDL's
philosophy of dynamic modules, we will ideally, at some point in the future, support multiple caches,
or even better, user-defined caches. The decision to be made is whether to provide infrastructure
now to plan for the future, or to implement a functional solution for now and extend it later.

To implement it now, the most straight-forward approach is to follow the way of the Command Service
and add a user service for each cache called a Cache Service that takes data from the materialization
service(s) over gRPC and stores it in a user-defined cache. This would mean creating a gRPC definition
for the cache service, and interfacing only with ElasticSearch for now, likely through the [official
client library][elasticsearch rust] for Rust, which would basically require copying of the source for
a Command Service.

To implement only communication with ElasticSearch would entail extending the materialization service(s)
to use the [elasticsearch] in place of a user-defined Cache Service, and to have slightly more error
handling in the materialization service(s) instead of the Cache Service. It would potentially be able
to determine earlier if failure occurred in storing projected data in the cache, but users would be able
to observe when materialization issues occur by looking at when the last update was made to the cached
data and comparing it to the normal rate of update.

The simpler approach is to leave more work for later, but it is more beneficial to the structure of
this project to separate it for a good separation of concerns and a more common structure across our
sub-applications.


## Configuration of Materialization

### What to Store

The main data to be stored is the definitions of the materialization to be done and how often it
should be recalculated. Views and Relations have already been defined above, so the following fields
are for configuration of materialization frequency:

Field       | Type | Description
-----------:|:----:|:-----------
eager       | bool | whether to recalculate on every update
minWaitTime | int  | minimum number of seconds to wait between updates
updateEvery | int  | number of seconds to update after when updates arrive

### Where

Though it would be possible to store the configuration information where it is needed (in the
materialization service(s)), we already have the schema registry and a configuration service (in
development), so there are more appropriate locations for configuration data to be stored. 

If the materialization is per-schema, then the obvious approach is to extend what a schema holds and
store materialization definitions there. However, for the metadata about when to update projections,
it makes more sense to move it to the configuration service like the metadata for schemas. For schema
data, we are making the distinction between business data and application behavior, which is JSON Schema
definitions and Kafka topics/AMQP exchanges, respectively.


## Test Plan

The simple use cases are:

- Eager materialization
- Caching of projected data
- Addition of materializer definitions
- Configuration of automatic projection timing

These cases can all be tested primarily with manual testing, with some unit tests advised.

The primary complex use cases are:

- Deeply-nested data
- Inter-related schema projection
- Proper cache updates when complex data updates

Though at least manual testing is required for these, at least some Python tests should be added
to ensure that this feature isn't accidentally broken by future feature additions.


[jmespath]: https://jmespath.org
[elasticsearch rust]: https://docs.rs/elasticsearch/7.10.1-alpha.1/elasticsearch/

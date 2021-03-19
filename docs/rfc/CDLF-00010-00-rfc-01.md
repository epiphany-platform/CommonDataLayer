# Front Matter

```
 Title: CDL Ingestion API versioning
 Author: Łukasz Biel
 Team: CDL
 Reviewer: CDLTeam
 Created on: 5/2/2021
 Last updated: 10/2/2021
 Tracking issue: https://github.com/epiphany-platform/CommonDataLayer/issues/225
```

# Introduction

We need to introduce a way to version the `CIM` schema.

## Summary

We should provide JSON schema with `CIM` format.
We need to return the most recent message version in case of deserialization failure in `DR`.

## Glossary

`CIM` - CDL Ingestion Message \
`DR` - data router

## Context

Recent problems on the Digital Enterprise side have yielded it necessary
to introduce `CIM` schema version tracking as a preliminary measure.

## Goals and Requirements

It should be transparent to CDL users which schema they have to use when inserting data into CDL.
In case of version mismatch, `DR` must send a notification with information about the mishap, or,
in the case of synchronous protocols, it must return a descriptive error.

# Solutions

## Existing solution

`CIM` schema versioning does not exist.
Users can find data format specifications in [data-router][data-router-readme].
It contains only mandatory fields, skipping optional, e.g., `orderGroupId`.
The most recent version of the data format is available when looking into the CDL code.

## Proposed solution

### 1. Publish schema, do not validate it

We will publish the `CIM` schema in `/docs/cdl_schema/vX.json` in JSON schema format.
The schema version will only consist of the `MAJOR` version number.
Furthermore, new versions will be published once we propose a new mandatory field.
We will be adding new optional fields to the previous `MAJOR` without a version bump.
`data-router` will not check the optional `version` field in the `CIM` message.
If the message fails to deserialize, `DR` will produce an error with a schema version that is compatible with it.

> If we cannot deserialize the message to the expected format, DataRouter cannot easily extract the `version` field.
We could parse payload as `JSONMap` and find a field named `version`, but all we'd gain is a more verbose error
message "DR supports version X, you sent us Y". While it, at the surface, seems better, in reality,
the user is aware of what version they are sending, and this extra step is unnecessary.

In the proposed scenario, `DR` can handle only one version of the schema.
Furthermore, multiple versions of the `data-router` may share one version of the schema.
DataRouter assumes that if the `version` field is missing, the payload is in supported data format ("most recent").

#### Comments & Questions

* Should we use JSON schema with MsgPack as well? In theory, we are only describing the format of `CIM`.
  However, what if `MsgPack` requires more info that `JSONschema` would be able to convey?
* Let's assume we document a feature, e.g., new field `xyz`.
  The user would send this field in payload expecting `feature` to work, but that depends on the `DR` he's running.
  If the user is running an older version of `DR`, it would quietly discard the given field.
  The whole premise of versioning is to ease debugging of errors encountered in CDL.
  Using only `MAJOR` seems to go against it. We may need to add a `MINOR` to the mix, or report extra fields.

### 2. Validate version, support multiple parsers

We will publish specifications in the same way as mentioned in the prior solution.
We will read the `version` field first (extracting it before parsing the whole JSON), and `DR` will choose the correct parser based on that version.
Internally all messages would be parsed to one unified format.
Such a feature will guarantee that the client can use new versions of CDL without upgrading his applications.
In this case, we should assume that if a message has no `version` field, it uses **oldest**, probably `1.0` schema.

#### Comments & Questions
* How are we going to approach deprecations in this case?
* Client still may send a too recent version of format to an old instance of CDL.
* Supporting many deserializers is problematic at best, so we need to be extra careful when introducing this.

### Other considerations

#### Headers
We could use headers to pass info about the version.
`GRPC`, `Kafka`, and `RabbitMQ` support headers (however, `RabbitMQ` rust clients have slight issues).
However, the header should not dictate the workings of a program. In the first proposal,
a header with a different version that current would cause an error,
in the second, only header with a too recent version.

# Test Plan
TBD

# Further considerations
## Impact on other teams
Depending on the solution, external teams have to familiarize themselves with schemas either
during each upgrade or when `DR` changelog explicitly states bump of `MAJOR` number in `CIM`.

# Tasks and timeline
TBD

[data-router-readme]: ../data_router.md

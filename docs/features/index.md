# Features

## Guidelines
* `name or description` field demonstrates only brief info about the feature itself, to learn more, please follow to the RFC in question.
* There may be multiple RFCs/documents for each feature, but usually only one will be linked here.
* Generally, the state of the feature looks should follow these guidelines:
  - `[discussion/idea/request]->[TechSpec/RFC] and optionally [PoC] -> [RC] -> [Ready]`
  - `RC` means that the feature is waiting for the release
  - `Ready` means that the feature is tested, merged and released
  - `Retired` or `Abandoned` means that the feature was dropped due to various factors, including but not limited to lack of support, legacy code or lack of usage.
  - `Suggestion` means the feature was requested or suggested, and is in preplanning or plannig stage. This shoud result in a Technical Specification or in a doc.
  - Features in `TechSpec` phase can have missing or broken document links, since documentation is usually waiting for pull request or have additional comments pending.

* in case there is a developmnent release we will mark it as XXX-rc, those are not fully fledged releases, a lot of the things can be broken there or undergoing testing
  this is basically a beta branch that may or may not result in stabilization and/or release. As such State for those features will remain `RC` until released to master

## Feature List
| Feature ID    | name or description                      | State      | CDL version | latestRFC                             |
|---------------|------------------------------------------|------------|-------------|---------------------------------------|
| CDLF-00001-00 | Basic Document Repository                | Retired    | 0.0.1       | N/A                                   |
| CDLF-00002-00 | Basic Binary Repository                  | Retired    | 0.0.1       | N/A                                   |
| CDLF-00003-00 | Basic Timeseries Repository              | Retired    | 0.0.1       | N/A                                   |
| CDLF-00004-00 | Query Service and Query Routing          | Ready      | 0.0.1       | N/A                                   |
| CDLF-00005-00 | Query Service: Postgres                  | Ready      | 0.0.1       | N/A                                   |
| CDLF-00006-00 | Automatic Query Destination              | Ready      | 0.0.1       | N/A                                   |
| CDLF-00007-00 | System Metrics Support                   | Ready      | 0.0.1       | N/A                                   |
| CDLF-00008-00 | Victoria Metrics Support                 | Ready      | 0.0.9       | N/A                                   |
| CDLF-00009-00 | CDL Input Message Batching               | Ready      | 1.0.0       | N/A                                   |
| CDLF-0000A-00 | Message Ordering                         | Ready      | 1.0.0       | [DOC](./ordering.md)                  |
| CDLF-0000B-00 | Access Groups                            | Suggestion | -----       | N/A                                   |
| CDLF-0000C-00 | Full GRPC communication support          | Ready      | 1.0.0       | [RFC](../rfc/CDLF-0000C-00-rfc-01.md) |
| CDLF-0000D-00 | Service Mesh (istio)                     | Abandoned  | -----       | N/A                                   |
| CDLF-0000E-00 | CDL Input Message Format - MessagePack   | TechSpec   | -----       | [RFC](../rfc/CDLF-0000E-00-rfc-01.md) |
| CDLF-0000F-00 | Configuration Service                    | Suggestion | -----       |                                       |
| CDLF-00010-00 | Protocol Versioning                      | TechSpec   | -----       | [RFC](../rfc/CDLF-00010-00-rfc-01.md) |
| CDLF-00011-00 | Basic Materialization                    | Abandoned  | -----       | [RFC](../rfc/CDLF-00011-00-rfc-01.md) |
| CDLF-00012-00 | Edge registry                            | Ready      | 1.0.0       | [RFC](../rfc/CDLF-00012-00-rfc-01.md) |
| CDLF-00013-00 | Materialized views                       | Ready      | 1.0.0       | [RFC](../rfc/CDLF-00013-00-rfc-01.md) |
| CDLF-00014-00 | Materialization - Filters                | Ready      | 1.0.0       | N/A                                   |
| CDLF-00016-00 | Schema-Registry-less CDL deployment      | Ready      | 1.0.0       | [RFC](../rfc/CDLF-00016-00-rfc-01.md) |
| CDLF-00017-00 | Materialization - Computation            | TechSpec   | -----       |                                       |
| CDLF-00018-00 | Materialization - Materialized Types     | TechSpec   | -----       |                                       |
| CDLF-00019-00 | Materialization - Notifications          | Ready      | 1.0.0       | N/A                                   |
| CDLF-0001A-00 | Object-side configuration                | Suggestion | -----       |                                       |
| CDLF-0001B-00 | CIM Object Valdiation                    | Suggestion | -----       |                                       |
| CDLF-0001C-00 | Materialization - Relationships          | Idea       | -----       |                                       |

## References:
[CDL RFC directory](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/docs/rfc)

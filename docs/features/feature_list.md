## Guidelines
* `name or description` field demonstrates only brief info about the feature itself, to learn more, please follow to the RFC in question.
* There may be multiple RFCs/documents for each feature, but usually only one will be linked here.
* Generally, the state of the feature looks should follow these guidelines:
  - `[discussion/idea/request]->[TechSpec/RFC] and optionally [PoC] -> [RC] -> [Ready]`
  - `RC` means that the feature is waiting for the release
  - `Ready` means that the feature is tested, merged and released
  - `Retired` means that the feature was dropped due to various factors, including but not limited to lack of support, legacy code or lack of usage.
  - Features in `TechSpec` phase can have missing or broken document links, since documentation is usually waiting for pull request or have additional comments pending.

## Feature List
| Feature ID    | name or description								|State    |CDL version| latest RFC |
|---------------|---------------------------------------------------|---------|-----------|------------|
| CDLF-00001-00 | Basic Document Repository                         |Retired  |0.0.1      | N/A        |
| CDLF-00002-00 | Basic Binary Repository                           |Retired  |0.0.1      | N/A
| CDLF-00003-00 | Basic Timeseries Repository                       |Retired  |0.0.1      | N/A
| CDLF-00004-00 | Query Service and Query Routing                   |Ready    |0.0.1      | N/A
| CDLF-00005-00 | Query Service: Postgres                           |Ready    |0.0.1      | N/A
| CDLF-00006-00 | Automatic Query Destination                       |Ready    |0.0.1      | N/A
| CDLF-00007-00 | System Metrics Support                            |Ready    |0.0.1      | N/A
| CDLF-00008-00 | Victoria Metrics Support                          |Ready    |0.0.9      | N/A
| CDLF-00009-00 | CDL Input Message Batching                        |RC       |-----      | N/A
| CDLF-0000A-00 | Message Ordering                                  |RC       |0.0.9      | [DOC](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/docs/features/message_ordering.md)
| CDLF-0000B-00 | Access Groups                                     |TechSpec |-----      |
| CDLF-0000C-00 | Full GRPC communication support                   |PoC      |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-0000C-00-rfc-01.md)
| CDLF-0000D-00 | Service Mesh (istio)                              |PoC      |-----      | N/A
| CDLF-0000E-00 | CIM MessagePack Format                            |TechSpec |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/docs/rfc/docs/rfc/CDLF-0000E-00-rfc-01.md)
| CDLF-0000F-00 | Configuration Service                             |TechSpec |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-0000F-00-rfc-01.md)
| CDLF-00010-00 | Protocol Versioning                               |TechSpec |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-00010-00-rfc-01.md)
| CDLF-00011-00 | Basic Materialization                             |TechSpec |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-00011-00-rfc-01.md)
| CDLF-00012-00 | Edge repository                                   |RFC      |-----      | [RFC](https://github.com/epiphany-platform/CommonDataLayer/blob/develop/docs/rfc/CDLF-00012-00-rfc-01.md)



## References:
[CDL RFC directory](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/docs/rfc)


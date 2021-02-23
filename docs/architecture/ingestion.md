# Ingestion Layer

Services in this layer are responsible for accepting generic messages from external systems via a message queue, validating them and and forwarding the message to correct repository.  
Currently consists only of the [Data Router][data-router]. The [Data Router][data-router] accepts messages in the following format:

```json
{
  "schemaId": "ca435cee-2944-41f7-94ff-d1b26e99ba48",
  "objectId": "fc0b95e1-07eb-4bf8-b691-1a85a49ef8f0",
  "data": { ...valid json object }
}
```

For more details, see the Data Router's [readme][data-router].

[data-router]: data_router.md

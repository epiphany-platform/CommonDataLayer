# Client routing

CDL allows users to bypass routing present in SR via custom routing table defined in `configuration.toml` files (more info at [configuration documentation](../configuration/index.md)).
This should be used at user's discretion - as routing will not be managed by CDL stack there's possibility of missing messages, or other issues that may arise from using this.
Nevertheless, if there's requirement of customized, static routing, this can be arranged via following steps:

## Routing data for QR and DR
Both QR and DR accept configuration section:

```toml
[repositories]
backup_data = { insert_destination = "cdl.document.none.data", query_address = "http://localhost:50202", repository_type = "DocumentStorage" }
```

It's a dictionary, where each entry is an object consisting of 3 fields:

* insert_destination - DR will route messages based on this field; it must use main `communication_method`
* query_address - QR will request data from QS located at given address
* repository_type - DocumentStorage or Timeseries; used by QR for querying

## Sending statically routed messages to cdl

For purpose of static routing, CDL accepts `options` object within CDL Input Message:

```json
{
    "objectId": "09a1048e-81dc-4286-821c-91d48086ce05",
    "schemaId": "9d111ba6-b855-41dc-9f91-227c2fdb4c18",
    "data": { "field": false, "id": 5 },
    "options": { "repositoryId": "backup_data" }
}
```

Within that object, there's optional field `repositoryId` that will tell DR to use it's value to lookup predefined `repositories`.
In above case, this will cause DR to route message to `cdl.document.none.data` topic (assuming `communication_method` is kafka).

## Querying statically routed messages

Some for QR, there's additional header on `single` and `multiple` routes: `REPOSITORY_ID`. You can use it to point QR to specific entry in routing table.


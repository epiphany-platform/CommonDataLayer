# CDL configuration (WIP)

Each application accepts a configuration file in `.toml` format.
Example configuration is present in this directory:

* [api](./api.md)
* [command-service](./command-service.md)
* [data-router](./data-router.md)
* [edge-registry](./edge-registry.md)
* [materializer-general](./materializer-general.md)
* [materializer-ondemand](./materializer-ondemand.md)
* [object-builder](./object-builder.md)
* [partial-update-engine](./partial-update-engine.md)
* [query-router](./query-router.md)
* [query-service](./query-service.md)
* [query-service-ts](./query-service-ts.md)
* [schema-registry](./schema-registry.md)

Configuration is loaded in order:

```
/etc/cdl/default.{ext}
/etc/cdl/{app-name}.{ext}
/etc/cdl/{env}/default.{ext}
/etc/cdl/{env}/{app-name}.{ext}

$HOME/.cdl/default.{ext}
$HOME/.cdl/{app-name}.{ext}
$HOME/.cdl/{env}/default.{ext}
$HOME/.cdl/{env}/{app-name}.{ext}

./.cdl/default.{ext}
./.cdl/{app-name}.{ext}
./.cdl/{env}/default.{ext}
./.cdl/{env}/{app-name}.{ext}

{custom}/default.{ext}
{custom}/{app-name}.{ext}
{custom}/{env}/default.{ext}
{custom}/{env}/{app-name}.{ext}

ENVs
```

`{env}` is environment variable `ENVIRONMENT`; default value `development`  
`{app-name}` is application name with dashes  
`{ext}` is file extension, currently only `.toml`  
`{custom}` is `CDL_CONFIG` environment variable  
`ENVs` are app own environment variables

Configs are merged top to bottom, so value declared in `./` overwrites `/etc/` and so on.
Environment variables supersede every config file.

`ENVs` are in format:
```
{APP-NAME}_{SETTING-PATH}
```

`SETTING-PATH` is structure path of each config option, separated by `__` - double underscore.

examples:

```
DATA_ROUTER_KAFKA__BROKERS
COMMAND_SERVICE_AMQP__CONSUME_OPTIONS__NO_LOCAL
QUERY_SERVICE_REPOSITORY_KIND
```

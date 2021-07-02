# Common Data Layer Testing Framework

Testing framework for [Common Data Layer][cdl] , implemented in the [.Net5][net5] language.

## Prerequisites

CDL and infrastructure have to be deployed somewhere. Please see our docs - [deployment][deployment]. 

| Env variable name | Value |
|---|---|
| CDL_COMMAND_SERVICE_ADDRESS | http://localhost:50202 |
| CDL_DATA_ROUTER_ADDRESS | http://localhost:50102 |
| CDL_EDGE_REGISTRY_ADDRESS | http://localhost:50110 |
| CDL_MATERIALIZER_GENERAL_ADDRESS | http://localhost:50203 |
| CDL_MATERIALIZER_ONDEMAND_ADDRESS | http://localhost:50108 |
| CDL_QUERY_ROUTER_ADDRESS | http://localhost:50103 |
| CDL_QUERY_SERVICE_ADDRESS | http://localhost:50201 |
| CDL_SCHEMA_REGISTRY_ADDRESS | http://localhost:50101 |
| CDL_KAFKA_BROKER | localhost:9092 |
| CDL_KAFKA_DATA_INPUT_TOPIC | cdl.data.input |
| CDL_KAFKA_EDGE_INPUT_TOPIC | cdl.edge.input |
| CDL_SCHEMA_REGISTRY_DESTINATION | cdl.document.1.data |

Before you start, you have to have latest proto files. Test solution requires proto files in 'proto' folder. To do that you can create symlink 
'ln -s ../crates/rpc/proto .'


## Launch tests
If you use sources 'dotnet test'

If you use build artifacts. 'dotnet vstest CDL.Tests.dll'

Also you can use 'build.sh', to build app and create docker image. Then you have to run container to lunch the tests.

[net5]: https://docs.microsoft.com/en-us/aspnet/core/?view=aspnetcore-5.0
[cdl]: https://epiphany-platform.github.io/CommonDataLayer/
[deployment]: https://epiphany-platform.github.io/CommonDataLayer/deployment/index.html
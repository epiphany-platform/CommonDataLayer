# Materializer - General

## Configuration (Environment variables)

### General:
| Name                 | Short Description                                 | Example                      | Mandatory  | Default |
|----------------------|---------------------------------------------------|------------------------------|------------|---------|
| INPUT_PORT           | gRPC server port                                  | `50110`                      | yes        | no      |
| METRICS_PORT         | Port to listen on for Prometheus metrics          | `58105`                      | no(default)| 58105   |
| STATUS_PORT          | Port exposing status of the application           | `3000`                       | no(default)| 3000    |
| OBJECT_BUILDER_ADDR  | Address of object builder (grpc)                  | `http://objectbuilder:50101` | yes        | no      |
| MATERIALIZER         | Type of materializer being used *                 | `postgres`                   | yes        | no      |

extra:
- Materializer types: postgres

### Configuration for Postgres Materializer:
| Name                 | Short Description                                 | Example                      | Mandatory  | Default |
|----------------------|---------------------------------------------------|------------------------------|------------|---------|
| POSTGRES_USERNAME    |  Postgres Username                                | `postgres`                   | yes        | no      |
| POSTGRES_PASSWORD    |  Postgres Password                                | `P422w0rd`                   | yes        | no      |
| POSTGRES_PORT        |  Postgres Port                                    | `5432`                       | no(default)| 5432    |
| POSTGRES_DBNAME      |  Postgres Database Name                           | `cdl`                        | yes        | no      |
| POSTGRES_SCHEMA      |  Postgres Schema Name                             | `public`                     | yes        | public  |


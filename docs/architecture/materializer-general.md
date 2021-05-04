# Materializer - General

### Configuration (Environment variables)

| Name                 | Short Description                                 | Example                      | Mandatory | Default |
|----------------------|---------------------------------------------------|------------------------------|-----------|---------|
| INPUT_PORT           | gRPC server port                                  | 50110                        | yes       | no      |
| METRICS_PORT         | Port to listen on for Prometheus metrics          | 58105                        | no        | 58105   |
| STATUS_PORT          | Port exposing status of the application           | 3000                         | no        | 3000    |
| OBJECT_BUILDER_ADDR  | Address of object builder (grpc)                  | http://objectbuilder:50101   | yes       | no      |
| MATERIALIZER         | Type of materializer being used *                 | postgres                     | yes       | no      |

extra:
- Materializer types: postgres

### Configuration for Postgres Materializer

| Name                 | Short Description                                 | Example                      | Mandatory | Default |
|----------------------|---------------------------------------------------|------------------------------|-----------|---------|
| postgres_username    |  Postgres Username                                | postgres                     | yes       | no      |
| postgres_password    |  Postgres Password                                | P422w0rd                     | no        | no      |
| postgres_port        |  Postgres Port                                    | 5432                         | no        | 5432    |
| postgres_dbname      |  Postgres Database Name                           | cdl                          | yes       | no      |
| postgres_schema      |  Postgres Schema Name                             | public                       | yes       | public  |


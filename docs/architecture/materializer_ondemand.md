# Materializer - On Demand

## Configuration (Environment variables)

| Name                 | Short Description                                 | Example                      | Mandatory  | Default |
|----------------------|---------------------------------------------------|------------------------------|------------|---------|
| INPUT_PORT           | gRPC server port                                  | 50110                        | yes        | no      |
| METRICS_PORT         | Port to listen on for Prometheus metrics          | 58105                        | no(default)| 58105   |
| STATUS_PORT          | Port exposing status of the application           | 3000                         | no(default)| 3000    |
| OBJECT_BUILDER_ADDR  | Address of object builder (grpc)                  | http://objectbuilder:50101   | yes        | no      |

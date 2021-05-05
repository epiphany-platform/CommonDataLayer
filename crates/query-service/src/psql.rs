use anyhow::Context;
use bb8_postgres::tokio_postgres::config::Config as PgConfig;
use bb8_postgres::tokio_postgres::{types::ToSql, NoTls, SimpleQueryMessage};
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    tokio_postgres::RowStream,
};
use clap::Clap;
use futures_util::TryStreamExt;
use rpc::query_service::{query_service_server::QueryService, ObjectStream};
use rpc::query_service::{Object, ObjectIds, RawStatement, SchemaId, ValueBytes};
use serde_json::Value;
use tonic::{Request, Response, Status};
use utils::{
    metrics::{self, counter},
    psql::validate_schema,
};
use uuid::Uuid;

#[derive(Debug, Clap)]
pub struct PsqlConfig {
    /// Postgres username
    #[clap(long, env = "POSTGRES_USERNAME")]
    username: String,
    /// Postgres password
    #[clap(long, env = "POSTGRES_PASSWORD")]
    password: String,
    /// Host of the postgres server
    #[clap(long, env = "POSTGRES_HOST")]
    host: String,
    /// Port on which postgres server listens
    #[clap(long, env = "POSTGRES_PORT", default_value = "5432")]
    port: u16,
    /// Database name
    #[clap(long, env = "POSTGRES_DBNAME")]
    dbname: String,
    /// SQL schema available for service
    #[clap(long, env = "POSTGRES_SCHEMA", default_value = "public")]
    schema: String,
}

pub struct PsqlQuery {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

impl PsqlQuery {
    pub async fn load(config: PsqlConfig) -> anyhow::Result<Self> {
        let mut pg_config = PgConfig::new();
        pg_config
            .user(&config.username)
            .password(&config.password)
            .host(&config.host)
            .port(config.port)
            .dbname(&config.dbname);
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = Pool::builder()
            .build(manager)
            .await
            .context("Failed to build connection pool")?;

        let schema = config.schema;

        if !validate_schema(&schema) {
            anyhow::bail!(
                "Schema `{}` has invalid name. It can contain only ascii letters, numbers and underscores",
                schema
            );
        }

        Ok(Self { pool, schema })
    }

    async fn connect(
        &self,
    ) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>, Status> {
        self.pool
            .get()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }

    async fn make_query(
        &self,
        query: &str,
        args: &[&(dyn ToSql + Sync)],
    ) -> Result<RowStream, Status> {
        let connection = self.connect().await?;
        let statement = connection.prepare(query).await.map_err(|err| {
            Status::internal(format!("Unable to prepare query statement: {}", err))
        })?;

        connection
            .query_raw(&statement, slice_iter(args))
            .await
            .map_err(|err| Status::internal(format!("Unable to query data: {}", err)))
    }

    fn map_id_payload_rows(stream: RowStream) -> ObjectStream<Status> {
        Box::pin(
            stream
                .map_ok(|row| {
                    let object_id = row.get::<usize, Uuid>(0).to_string();
                    let payload = row.get::<usize, Value>(1).to_string().into_bytes();
                    Object { object_id, payload }
                })
                .map_err(|err| Status::internal(format!("Unable to query data: {}", err))),
        )
    }

    fn collect_simple_query_messages(messages: Vec<SimpleQueryMessage>) -> Result<Vec<u8>, String> {
        let data: Vec<Vec<String>> = messages
            .into_iter()
            .map(|msg| match msg {
                SimpleQueryMessage::Row(row) => {
                    let mut fields: Vec<String> = Vec::new();
                    for i in 0..row.len() {
                        fields.push(
                            row.try_get(i)
                                .map_err(|e| {
                                    format!("Error getting data from row at column {}: {}", i, e)
                                })?
                                .unwrap_or("")
                                .to_string(),
                        );
                    }
                    Ok(fields)
                }
                SimpleQueryMessage::CommandComplete(command) => Ok(vec![command.to_string()]),
                _ => Err("Could not match SimpleQueryMessage".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?;

        serde_json::to_vec(&data).map_err(|e| format!("Error serializing data to json: {}", e))
    }
}

#[tonic::async_trait]
impl QueryService for PsqlQuery {
    type QueryMultipleStream = ObjectStream<tonic::Status>;

    #[tracing::instrument(skip(self))]
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<Self::QueryMultipleStream>, Status> {
        let request = request.into_inner();

        counter!("cdl.query-service.query-multiple.psql", 1);

        let object_ids: Vec<Uuid> = request
            .object_ids
            .into_iter()
            .map(|id| id.parse::<Uuid>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let query_str = format!(
            "SELECT d.object_id, d.payload \
                 FROM (\
                     SELECT object_id, max(version) as max \
                     FROM {}.data \
                     WHERE object_id = ANY($1) \
                     GROUP BY object_id\
                 ) maxes \
                 JOIN {}.data d \
                 ON d.object_id = maxes.object_id AND d.version = maxes.max",
            self.schema, self.schema
        );

        let row_stream = self
            .make_query(query_str.as_str(), &[&object_ids.as_slice()])
            .await?;

        let stream = Self::map_id_payload_rows(row_stream);
        Ok(tonic::Response::new(stream))
    }

    type QueryBySchemaStream = ObjectStream<tonic::Status>;

    #[tracing::instrument(skip(self))]
    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<Self::QueryBySchemaStream>, Status> {
        let request = request.into_inner();

        counter!("cdl.query-service.query-by-schema.psql", 1);

        let schema_id = request
            .schema_id
            .parse::<Uuid>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let query_str = format!(
            "SELECT object_id, payload \
                 FROM {}.data d1 \
                 WHERE schema_id = $1 AND d1.version = (\
                     SELECT MAX(version) \
                     FROM {}.data d2 \
                     WHERE d2.object_id = d1.object_id\
                 )",
            self.schema, self.schema
        );

        let row_stream = self.make_query(query_str.as_str(), &[&schema_id]).await?;

        let stream = Self::map_id_payload_rows(row_stream);
        Ok(tonic::Response::new(stream))
    }

    #[tracing::instrument(skip(self))]
    async fn query_raw(
        &self,
        request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query_raw.psql", 1);
        let connection = self.connect().await?;
        let messages = connection
            .simple_query(request.into_inner().raw_statement.as_str())
            .await
            .map_err(|err| Status::internal(format!("Unable to query_raw data: {}", err)))?;

        Ok(tonic::Response::new(ValueBytes {
            value_bytes: Self::collect_simple_query_messages(messages).map_err(|err| {
                Status::internal(format!(
                    "Unable to collect simple query messages data: {}",
                    err
                ))
            })?,
        }))
    }
}

fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
